#![allow(non_snake_case, non_camel_case_types, dead_code)]

use std::ptr;

use winapi::shared::windef::{HWND, RECT};
use winapi::shared::guiddef::{GUID, REFIID};
use winapi::shared::winerror::{HRESULT, S_OK, S_FALSE, E_NOINTERFACE, E_FAIL};
use winapi::shared::ntdef::ULONG;

// ── IUnknown ──────────────────────────────────────────────────

#[repr(C)]
pub struct IUnknownVtable {
    pub QueryInterface: unsafe extern "system" fn(this: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(this: *mut IUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(this: *mut IUnknown) -> ULONG,
}

#[repr(C)]
pub struct IUnknown {
    pub vtbl: *const IUnknownVtable,
}

impl IUnknown {
    pub unsafe fn release(&self) -> ULONG {
        ((*self.vtbl).Release)(self as *const _ as *mut _)
    }
}

// ── IInitializeWithFile ───────────────────────────────────────
// {b7d14566-0509-4cce-a71f-0a554233bd9b}

#[repr(C)]
pub struct IInitializeWithFileVtable {
    // IUnknown base
    pub QueryInterface: unsafe extern "system" fn(this: *mut IInitializeWithFile, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(this: *mut IInitializeWithFile) -> ULONG,
    pub Release: unsafe extern "system" fn(this: *mut IInitializeWithFile) -> ULONG,
    // IInitializeWithFile
    pub Initialize: unsafe extern "system" fn(this: *mut IInitializeWithFile, pszFilePath: *const u16, grfMode: u32) -> HRESULT,
}

#[repr(C)]
pub struct IInitializeWithFile {
    pub vtbl: *const IInitializeWithFileVtable,
}

impl IInitializeWithFile {
    pub unsafe fn initialize(&self, path: *const u16, mode: u32) -> HRESULT {
        ((*self.vtbl).Initialize)(self as *const _ as *mut _, path, mode)
    }

    pub unsafe fn release(&self) -> ULONG {
        ((*self.vtbl).Release)(self as *const _ as *mut _)
    }
}

// ── IPreviewHandler ───────────────────────────────────────────
// {8895b1c6-b41f-4c1c-a562-0d564250836f}

#[repr(C)]
pub struct IPreviewHandlerVtable {
    // IUnknown base
    pub QueryInterface: unsafe extern "system" fn(this: *mut IPreviewHandler, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(this: *mut IPreviewHandler) -> ULONG,
    pub Release: unsafe extern "system" fn(this: *mut IPreviewHandler) -> ULONG,
    // IPreviewHandler
    pub SetWindow: unsafe extern "system" fn(this: *mut IPreviewHandler, hwnd: HWND, prc: *const RECT) -> HRESULT,
    pub SetRect: unsafe extern "system" fn(this: *mut IPreviewHandler, prc: *const RECT) -> HRESULT,
    pub DoPreview: unsafe extern "system" fn(this: *mut IPreviewHandler) -> HRESULT,
    pub Unload: unsafe extern "system" fn(this: *mut IPreviewHandler),
    pub SetFocus: unsafe extern "system" fn(this: *mut IPreviewHandler) -> HRESULT,
    pub QueryFocus: unsafe extern "system" fn(this: *mut IPreviewHandler, phwnd: *mut HWND) -> HRESULT,
    // TranslateAccelerator (we pass NULL/none)
    pub TranslateAccelerator: unsafe extern "system" fn(this: *mut IPreviewHandler, pmsg: *const core::ffi::c_void) -> HRESULT,
}

#[repr(C)]
pub struct IPreviewHandler {
    pub vtbl: *const IPreviewHandlerVtable,
}

impl IPreviewHandler {
    pub unsafe fn query_interface(&self, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
        ((*self.vtbl).QueryInterface)(self as *const _ as *mut _, riid, ppv)
    }

    pub unsafe fn set_window(&self, hwnd: HWND, prc: *const RECT) -> HRESULT {
        ((*self.vtbl).SetWindow)(self as *const _ as *mut _, hwnd, prc)
    }

    pub unsafe fn set_rect(&self, prc: *const RECT) -> HRESULT {
        ((*self.vtbl).SetRect)(self as *const _ as *mut _, prc)
    }

    pub unsafe fn do_preview(&self) -> HRESULT {
        ((*self.vtbl).DoPreview)(self as *const _ as *mut _)
    }

    pub unsafe fn unload(&self) {
        ((*self.vtbl).Unload)(self as *const _ as *mut _)
    }

    pub unsafe fn release(&self) -> ULONG {
        ((*self.vtbl).Release)(self as *const _ as *mut _)
    }
}

// ── IPreviewHandlerVisuals ────────────────────────────────────
// {196bf9a5-b346-4ef0-aa1e-5dcdb76768b1}

#[repr(C)]
pub struct IPreviewHandlerVisualsVtable {
    pub QueryInterface: unsafe extern "system" fn(this: *mut IPreviewHandlerVisuals, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(this: *mut IPreviewHandlerVisuals) -> ULONG,
    pub Release: unsafe extern "system" fn(this: *mut IPreviewHandlerVisuals) -> ULONG,
    pub SetBackgroundColor: unsafe extern "system" fn(this: *mut IPreviewHandlerVisuals, color: u32) -> HRESULT,
    pub SetFont: unsafe extern "system" fn(this: *mut IPreviewHandlerVisuals, plf: *const core::ffi::c_void) -> HRESULT,
    pub SetTextColor: unsafe extern "system" fn(this: *mut IPreviewHandlerVisuals, color: u32) -> HRESULT,
}

#[repr(C)]
pub struct IPreviewHandlerVisuals {
    pub vtbl: *const IPreviewHandlerVisualsVtable,
}

// ── IObjectWithSite ───────────────────────────────────────────
// {fc4801a3-2ba9-11cf-a229-00aa003d7352}

#[repr(C)]
pub struct IObjectWithSiteVtable {
    pub QueryInterface: unsafe extern "system" fn(this: *mut IObjectWithSite, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(this: *mut IObjectWithSite) -> ULONG,
    pub Release: unsafe extern "system" fn(this: *mut IObjectWithSite) -> ULONG,
    pub SetSite: unsafe extern "system" fn(this: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT,
    pub GetSite: unsafe extern "system" fn(this: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut *mut core::ffi::c_void) -> HRESULT,
}

#[repr(C)]
pub struct IObjectWithSite {
    pub vtbl: *const IObjectWithSiteVtable,
}

impl IObjectWithSite {
    pub unsafe fn set_site(&self, site: *mut IUnknown) -> HRESULT {
        ((*self.vtbl).SetSite)(self as *const _ as *mut _, site)
    }

    pub unsafe fn release(&self) -> ULONG {
        ((*self.vtbl).Release)(self as *const _ as *mut _)
    }
}

// ── IOleWindow ────────────────────────────────────────────────
// {00000114-0000-0000-C000-000000000046}

#[repr(C)]
pub struct IOleWindowVtable {
    pub QueryInterface: unsafe extern "system" fn(this: *mut IOleWindow, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(this: *mut IOleWindow) -> ULONG,
    pub Release: unsafe extern "system" fn(this: *mut IOleWindow) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(this: *mut IOleWindow, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(this: *mut IOleWindow, fEnterMode: i32) -> HRESULT,
}

#[repr(C)]
pub struct IOleWindow {
    pub vtbl: *const IOleWindowVtable,
}

impl IOleWindow {
    pub unsafe fn get_window(&self, phwnd: *mut HWND) -> HRESULT {
        ((*self.vtbl).GetWindow)(self as *const _ as *mut _, phwnd)
    }

    pub unsafe fn release(&self) -> ULONG {
        ((*self.vtbl).Release)(self as *const _ as *mut _)
    }
}

// ── Preview Handler Site Object ───────────────────────────────
// Implements IUnknown + IOleWindow + IObjectWithSite
// Some Preview Handlers need this to get the host window handle

pub const CLSID_IOLEWINDOW: GUID = GUID {
    Data1: 0x00000114,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

pub const CLSID_IOBJECTWITHSITE: GUID = GUID {
    Data1: 0xfc4801a3,
    Data2: 0x2ba9,
    Data3: 0x11cf,
    Data4: [0xa2, 0x29, 0x00, 0xaa, 0x00, 0x3d, 0x73, 0x52],
};

#[repr(C)]
pub struct PreviewHandlerSite {
    pub ole_window_vtbl: *const IOleWindowVtable,
    pub ref_count: ULONG,
    pub hwnd: HWND,
}

// 静态 vtable 实例
static OLE_WINDOW_VTABLE: IOleWindowVtable = IOleWindowVtable {
    QueryInterface: site_query_interface,
    AddRef: site_add_ref,
    Release: site_release_vtable,
    GetWindow: site_get_window,
    ContextSensitiveHelp: site_context_sensitive_help,
};

impl PreviewHandlerSite {
    pub unsafe fn new(hwnd: HWND) -> *mut PreviewHandlerSite {
        let site = Box::new(PreviewHandlerSite {
            ole_window_vtbl: &OLE_WINDOW_VTABLE,
            ref_count: 1,
            hwnd,
        });
        Box::into_raw(site)
    }
}

unsafe extern "system" fn site_query_interface(
    this: *mut IOleWindow,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_FAIL;
    }

    let site = this as *mut PreviewHandlerSite;
    let iid = &*riid;

    // IUnknown
    if iid.Data1 == 0x00000000
        && iid.Data2 == 0x0000
        && iid.Data3 == 0x0000
        && iid.Data4 == [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46]
    {
        *ppv = site as *mut _;
        site_add_ref(this);
        return S_OK;
    }

    // IOleWindow {00000114-0000-0000-C000-000000000046}
    if iid.Data1 == 0x00000114
        && iid.Data2 == 0x0000
        && iid.Data3 == 0x0000
        && iid.Data4 == [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46]
    {
        *ppv = this as *mut _;
        site_add_ref(this);
        return S_OK;
    }

    // 不响应 IObjectWithSite（vtable 布局不同，需要单独实现）

    *ppv = ptr::null_mut();
    E_NOINTERFACE
}

unsafe extern "system" fn site_add_ref(this: *mut IOleWindow) -> ULONG {
    let site = this as *mut PreviewHandlerSite;
    (*site).ref_count += 1;
    (*site).ref_count
}

pub unsafe extern "system" fn site_release(this: *mut PreviewHandlerSite) -> ULONG {
    (*this).ref_count -= 1;
    let count = (*this).ref_count;
    if count == 0 {
        drop(Box::from_raw(this));
    }
    count
}

// Vtable-compatible release that takes *mut IOleWindow
unsafe extern "system" fn site_release_vtable(this: *mut IOleWindow) -> ULONG {
    site_release(this as *mut PreviewHandlerSite)
}

unsafe extern "system" fn site_get_window(this: *mut IOleWindow, phwnd: *mut HWND) -> HRESULT {
    let site = this as *mut PreviewHandlerSite;
    if phwnd.is_null() {
        return E_FAIL;
    }
    *phwnd = (*site).hwnd;
    S_OK
}

unsafe extern "system" fn site_context_sensitive_help(_this: *mut IOleWindow, _fEnterMode: i32) -> HRESULT {
    S_FALSE
}
