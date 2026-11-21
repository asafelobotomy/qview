//! Application setup and Qt's picture work, in C++ (see `cpp/app.cpp`).

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("uqview/cpp/app.h");

        fn uqview_describe_application(version: &str);
        fn uqview_has_window() -> bool;
        fn uqview_suffixes() -> String;
        fn uqview_image_info(path: &str) -> String;
        fn uqview_copy_image(path: &str) -> bool;
        fn uqview_render_markup(src: &str, marks: &str, dst: &str, clipboard: bool) -> String;
        fn uqview_register_types();
        fn uqview_set_icon_theme(name: &str);
        fn uqview_forward_to_running(request: &str) -> bool;
    }
}
