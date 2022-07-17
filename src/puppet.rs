/*
    Copyright © 2022, Inochi2D Project
    Distributed under the 2-Clause BSD License, see LICENSE file.

    Authors: Aki "lethalbit" Van Ness
*/

use std::{path::PathBuf, marker::PhantomData};

#[cfg(feature = "logging")]
use tracing::debug;

use crate::{
    ffi::{inErrorGet, types::InPuppet},
    Result, core::Inochi2D,
};

#[cfg(feature = "opengl")]
use crate::ffi::inPuppetDraw;
use crate::ffi::{
    inPuppetDestroy, inPuppetGetName, inPuppetLoad, inPuppetLoadEx, inPuppetLoadFromMemory,
    inPuppetUpdate, types::InPuppetPtr,
};

pub struct Inochi2DPuppet<'a> {
    handle: InPuppetPtr,
    pub name: String,
    instance_lifetime: PhantomData<&'a ()>
}

impl<'a> Inochi2DPuppet<'a> {
    pub fn from_raw_handle(instance_lifetime: PhantomData<&'a ()>, handle: *mut InPuppet, name: String) -> Result<Self> {
        if handle.is_null() {
            let error_information = unsafe { inErrorGet() };

            if error_information.is_null() {
                Err("Unknown error (are C bindings valid?)".into())
            } else {
                let error_res = unsafe { String::try_from(*error_information) };

                Err(error_res.unwrap_or_else(|_| "Unknown error (UTF-8 decoding failed!)".into()))
            }
        } else {
            Ok(Inochi2DPuppet { handle, name, instance_lifetime })
        }
    }

    pub unsafe fn from_raw_parts(
        instance: &'a Inochi2D,
        buffer: *const u8,
        size: usize,
        name: Option<String>,
    ) -> Result<Self> {
        #[cfg(feature = "logging")]
        debug!("Constructing puppet from {} bytes", size);
        let hndl = unsafe { inPuppetLoadFromMemory(buffer, size) };

        Self::from_raw_handle(instance.instance_lifetime, hndl, name.unwrap_or(String::from("<in-memory-puppet>")))
    }

    pub(crate) fn new_from_raw_lifetime(instance_lifetime: PhantomData<&'a ()>, puppet: PathBuf) -> Result<Self> {
        let puppet_path = String::from(puppet.to_str().expect("Unable to get puppet path"));
        #[cfg(feature = "logging")]
        debug!("Constructing puppet from file {}", puppet_path);
        let hndl = unsafe { inPuppetLoadEx(puppet_path.as_ptr(), puppet_path.len()) };

        Self::from_raw_handle(instance_lifetime, hndl, puppet_path)
    }

    pub fn new(instance: &'a Inochi2D, puppet: PathBuf) -> Result<Self> {
        Self::new_from_raw_lifetime(instance.instance_lifetime, puppet)
    }

    pub fn update(&mut self) {
        #[cfg(feature = "logging")]
        debug!("Updating puppet {}", self.name);

        unsafe {
            inPuppetUpdate(self.handle);
        }
    }

    #[cfg(feature = "opengl")]
    pub fn draw(&mut self) {
        #[cfg(feature = "logging")]
        debug!("Drawing puppet {}", self.name);

        unsafe {
            inPuppetDraw(self.handle);
        }
    }
}

impl<'a> Drop for Inochi2DPuppet<'a> {
    fn drop(&mut self) {
        #[cfg(feature = "logging")]
        debug!("Disposing of puppet {}", self.name);

        unsafe {
            inPuppetDestroy(self.handle);
        }
    }
}
