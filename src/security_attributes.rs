// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.

use error::Error;

use std::marker::PhantomData;
use std::ptr::NonNull;

use winapi::shared::minwindef::{BOOL, DWORD, LPVOID};
use winapi::um::{
    accctrl::{
        ACCESS_MODE, EXPLICIT_ACCESS_W, TRUSTEE_IS_IMPERSONATE, TRUSTEE_IS_NAME, TRUSTEE_IS_SID,
        TRUSTEE_W,
    },
    aclapi::SetEntriesInAclW,
    minwinbase::{LPTR, SECURITY_ATTRIBUTES},
    securitybaseapi::{
        CreateWellKnownSid, GetSidLengthRequired, GetSidSubAuthority, GetSidSubAuthorityCount,
        InitializeSecurityDescriptor, InitializeSid, IsValidAcl, IsValidSid,
        SetSecurityDescriptorDacl, SetSecurityDescriptorGroup, SetSecurityDescriptorOwner,
        SetSecurityDescriptorSacl,
    },
    winbase::{LocalAlloc, LocalFree},
    winnt::{
        WinBuiltinAdministratorsSid, WinWorldSid, ACCESS_MASK, ACL, SECURITY_DESCRIPTOR,
        SECURITY_DESCRIPTOR_MIN_LENGTH, SECURITY_DESCRIPTOR_REVISION, SECURITY_MAX_SID_SIZE, SID,
        SID_IDENTIFIER_AUTHORITY, WELL_KNOWN_SID_TYPE,
    },
};

pub struct SecurityAttributes<'a> {
    pub descriptor: Option<&'a SecurityDescriptor<'a>>,
    pub inherit_handle: bool,
}

impl<'a> SecurityAttributes<'a> {
    pub fn new(
        descriptor: Option<&'a SecurityDescriptor<'a>>,
        inherit_handle: bool,
    ) -> SecurityAttributes<'a> {
        SecurityAttributes {
            descriptor,
            inherit_handle,
        }
    }

    pub fn get_raw(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD,
            lpSecurityDescriptor: self
                .descriptor
                .as_ref()
                .map(|desc| desc.ptr.as_ptr() as LPVOID)
                .unwrap_or(std::ptr::null_mut()),
            bInheritHandle: self.inherit_handle as BOOL,
        }
    }
}

pub struct SecurityDescriptor<'a> {
    ptr: NonNull<SECURITY_DESCRIPTOR>,
    acl_marker: PhantomData<&'a Acl<'a>>,
    sid_marker: PhantomData<&'a Sid>,
}

impl<'a> SecurityDescriptor<'a> {
    pub fn empty() -> Result<Self, Error> {
        unsafe {
            let p_sd = LocalAlloc(LPTR, SECURITY_DESCRIPTOR_MIN_LENGTH);
            if p_sd.is_null() {
                return Err(Error::last());
            }
            if InitializeSecurityDescriptor(p_sd, SECURITY_DESCRIPTOR_REVISION) == 0 {
                let err = Error::last();
                LocalFree(p_sd);
                return Err(err);
            }
            Ok(SecurityDescriptor {
                ptr: NonNull::new_unchecked(p_sd as _),
                acl_marker: PhantomData,
                sid_marker: PhantomData,
            })
        }
    }

    pub fn set_dacl(&mut self, acl: &'a Acl) -> Result<(), Error> {
        unsafe {
            if SetSecurityDescriptorDacl(self.ptr.as_ptr() as _, 1, acl.ptr.as_ptr(), 0) == 0 {
                return Err(Error::last());
            }
            Ok(())
        }
    }

    pub fn set_sacl(&mut self, acl: &'a Acl) -> Result<(), Error> {
        unsafe {
            if SetSecurityDescriptorSacl(self.ptr.as_ptr() as _, 1, acl.ptr.as_ptr(), 0) == 0 {
                return Err(Error::last());
            }
            Ok(())
        }
    }

    pub fn set_owner(&mut self, sid: &'a Sid) -> Result<(), Error> {
        unsafe {
            if SetSecurityDescriptorOwner(self.ptr.as_ptr() as _, sid.ptr.as_ptr() as _, 0) == 0 {
                return Err(Error::last());
            }
            Ok(())
        }
    }

    pub fn set_group(&mut self, sid: &'a Sid) -> Result<(), Error> {
        unsafe {
            if SetSecurityDescriptorGroup(self.ptr.as_ptr() as _, sid.ptr.as_ptr() as _, 0) == 0 {
                return Err(Error::last());
            }
            Ok(())
        }
    }
}

impl<'a> Drop for SecurityDescriptor<'a> {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.ptr.as_ptr() as LPVOID);
        }
    }
}

#[repr(transparent)]
pub struct Acl<'s> {
    ptr: NonNull<ACL>,
    sid_marker: PhantomData<&'s Sid>,
}

impl<'s> Acl<'s> {
    pub fn from_entries(
        entries: &[ExplicitAccess<'s, '_>],
        old_acl: Option<&Acl<'s>>,
    ) -> Result<Self, Error> {
        assert!(entries.len() < std::u32::MAX as usize);
        let mut new_acl = std::ptr::null_mut();
        let result = unsafe {
            SetEntriesInAclW(
                entries.len() as u32,
                entries.as_ptr() as *const _ as *mut _,
                old_acl
                    .map(|a| a.ptr.as_ptr())
                    .unwrap_or(std::ptr::null_mut()),
                &mut new_acl,
            )
        };
        if result == 0 {
            Ok(Acl {
                ptr: NonNull::new(new_acl).unwrap(),
                sid_marker: PhantomData,
            })
        } else {
            Err(Error(result))
        }
    }

    pub fn is_valid(&self) -> bool {
        unsafe { IsValidAcl(self.ptr.as_ptr() as _) != 0 }
    }
}

impl<'s> Drop for Acl<'s> {
    fn drop(&mut self) {
        unsafe { LocalFree(self.ptr.as_ptr() as _) };
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ExplicitAccess<'s, 't> {
    ea: EXPLICIT_ACCESS_W,
    trustee_marker: PhantomData<&'t Trustee<'s, 't>>,
}

impl<'s, 't> ExplicitAccess<'s, 't> {
    pub fn new() -> Self {
        unsafe { std::mem::zeroed() }
    }

    pub fn with_access_permissions(mut self, flags: ACCESS_MASK) -> Self {
        self.ea.grfAccessPermissions = flags;
        self
    }

    pub fn with_access_mode(mut self, flags: ACCESS_MODE) -> Self {
        self.ea.grfAccessMode = flags;
        self
    }

    pub fn with_inheritance(mut self, flags: DWORD) -> Self {
        self.ea.grfInheritance = flags;
        self
    }

    pub fn with_trustee(mut self, trustee: Trustee<'s, 't>) -> Self {
        self.ea.Trustee = trustee.trustee;
        self
    }

    pub fn with_sid_trustee(self, trustee_type: u32, sid: &'s Sid) -> Self {
        self.with_trustee(Trustee::new().with_type(trustee_type).with_sid(sid))
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Trustee<'s, 't> {
    trustee: TRUSTEE_W,
    trustee_marker: PhantomData<&'t Trustee<'s, 't>>,
    sid_marker: PhantomData<&'s Sid>,
}

impl<'s, 't> Trustee<'s, 't> {
    pub fn new() -> Self {
        unsafe { std::mem::zeroed() }
    }

    pub fn with_multiple(mut self, trustee: &'t Trustee<'s, 't>) -> Self {
        self.trustee.MultipleTrusteeOperation = TRUSTEE_IS_IMPERSONATE;
        // It doesn't actually get mutated but urgh X3
        self.trustee.pMultipleTrustee = (&trustee.trustee) as *const _ as *mut _;
        self
    }

    pub fn with_type(mut self, kind: u32) -> Self {
        self.trustee.TrusteeType = kind;
        self
    }

    pub fn with_name(mut self, name: &'s [u16]) -> Self {
        assert!(name.contains(&0));
        self.trustee.TrusteeForm = TRUSTEE_IS_NAME;
        self.trustee.ptstrName = name.as_ptr() as *mut _;
        self
    }

    pub fn with_sid(mut self, sid: &'s Sid) -> Self {
        self.trustee.TrusteeForm = TRUSTEE_IS_SID;
        self.trustee.ptstrName = sid.ptr.as_ptr() as *mut _;
        self
    }
}

pub struct Sid {
    ptr: NonNull<SID>,
}

impl Sid {
    pub fn new(
        authority: &SID_IDENTIFIER_AUTHORITY,
        sub_authorities: &[u32],
    ) -> Result<Self, Error> {
        assert!(sub_authorities.len() <= 8);
        unsafe {
            let auth_count = sub_authorities.len() as u8;
            let size = GetSidLengthRequired(auth_count);
            let psid = LocalAlloc(LPTR, size as usize);
            if psid.is_null() {
                return Err(Error::last());
            }

            if InitializeSid(psid, authority as *const _ as *mut _, auth_count) == 0 {
                let err = Error::last();
                LocalFree(psid);
                return Err(err);
            }

            let mut sid = Sid {
                ptr: NonNull::new_unchecked(psid as _),
            };

            for (i, &val) in sub_authorities.iter().enumerate() {
                *sid.sub_authority(i as u8) = val;
            }

            Ok(sid)
        }
    }

    pub fn well_known(sid_type: WELL_KNOWN_SID_TYPE, domain: Option<&Sid>) -> Result<Self, Error> {
        unsafe {
            let mut size = SECURITY_MAX_SID_SIZE as u32;
            let psid = LocalAlloc(LPTR, size as usize);
            if psid.is_null() {
                return Err(Error::last());
            }

            let result = CreateWellKnownSid(
                sid_type,
                domain
                    .map(|d| d.ptr.as_ptr() as LPVOID)
                    .unwrap_or(std::ptr::null_mut()),
                psid,
                &mut size,
            );
            if result == 0 {
                let err = Error::last();
                LocalFree(psid);
                return Err(err);
            }

            Ok(Sid {
                ptr: NonNull::new_unchecked(psid as _),
            })
        }
    }

    pub fn everyone() -> Result<Self, Error> {
        Sid::well_known(WinWorldSid, None)
    }

    pub fn admin_group() -> Result<Self, Error> {
        Sid::well_known(WinBuiltinAdministratorsSid, None)
    }

    pub fn is_valid(&self) -> bool {
        unsafe { IsValidSid(self.ptr.as_ptr() as _) != 0 }
    }

    pub fn sub_authority_count(&self) -> u8 {
        assert!(self.is_valid());
        unsafe { *GetSidSubAuthorityCount(self.ptr.as_ptr() as _) }
    }

    pub fn sub_authority(&mut self, index: u8) -> &mut u32 {
        assert!(index < self.sub_authority_count());
        unsafe { &mut *GetSidSubAuthority(self.ptr.as_ptr() as _, index as u32) }
    }
}

impl Drop for Sid {
    fn drop(&mut self) {
        unsafe { LocalFree(self.ptr.as_ptr() as _) };
    }
}

#[cfg(test)]
#[test]
// Based on the example from
// https://docs.microsoft.com/en-us/windows/desktop/SecAuthZ/creating-a-security-descriptor-for-a-new-object-in-c--
fn create_basic_sa() -> Result<(), Error> {
    use winapi::um::accctrl::{SET_ACCESS, TRUSTEE_IS_GROUP, TRUSTEE_IS_WELL_KNOWN_GROUP};
    use winapi::um::winnt::{KEY_ALL_ACCESS, KEY_READ};

    let everyone_sid = Sid::everyone()?;
    let admin_sid = Sid::admin_group()?;

    let access = [
        // Read access to Everyone
        ExplicitAccess::new()
            .with_access_mode(SET_ACCESS)
            .with_access_permissions(KEY_READ)
            .with_sid_trustee(TRUSTEE_IS_WELL_KNOWN_GROUP, &everyone_sid),
        // All access to Admins
        ExplicitAccess::new()
            .with_access_mode(SET_ACCESS)
            .with_access_permissions(KEY_ALL_ACCESS)
            .with_sid_trustee(TRUSTEE_IS_GROUP, &admin_sid),
    ];

    let acl = Acl::from_entries(&access, None)?;

    let mut sd = SecurityDescriptor::empty()?;
    sd.set_dacl(&acl)?;

    let sa = SecurityAttributes::new(Some(&sd), false);

    let _attrs: SECURITY_ATTRIBUTES = sa.get_raw();
    // Do something with _attrs
    Ok(())
}
