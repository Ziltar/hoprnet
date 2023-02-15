pub trait IContentPrefetcherTaskTrigger_Impl: Sized {
    fn TriggerContentPrefetcherTask(&self, packagefullname: &::windows::core::PCWSTR) -> ::windows::core::Result<()>;
    fn IsRegisteredForContentPrefetch(&self, packagefullname: &::windows::core::PCWSTR) -> ::windows::core::Result<u8>;
}
impl ::windows::core::RuntimeName for IContentPrefetcherTaskTrigger {
    const NAME: &'static str = "";
}
impl IContentPrefetcherTaskTrigger_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: IContentPrefetcherTaskTrigger_Impl, const OFFSET: isize>() -> IContentPrefetcherTaskTrigger_Vtbl {
        unsafe extern "system" fn TriggerContentPrefetcherTask<Identity: ::windows::core::IUnknownImpl, Impl: IContentPrefetcherTaskTrigger_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, packagefullname: ::windows::core::PCWSTR) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).TriggerContentPrefetcherTask(::core::mem::transmute(&packagefullname)).into()
        }
        unsafe extern "system" fn IsRegisteredForContentPrefetch<Identity: ::windows::core::IUnknownImpl, Impl: IContentPrefetcherTaskTrigger_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, packagefullname: ::windows::core::PCWSTR, isregistered: *mut u8) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).IsRegisteredForContentPrefetch(::core::mem::transmute(&packagefullname)) {
                ::core::result::Result::Ok(ok__) => {
                    *isregistered = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ::windows::core::IInspectableVtbl::new::<Identity, IContentPrefetcherTaskTrigger, OFFSET>(),
            TriggerContentPrefetcherTask: TriggerContentPrefetcherTask::<Identity, Impl, OFFSET>,
            IsRegisteredForContentPrefetch: IsRegisteredForContentPrefetch::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<IContentPrefetcherTaskTrigger as ::windows::core::Interface>::IID
    }
}
