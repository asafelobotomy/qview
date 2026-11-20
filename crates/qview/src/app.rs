//! Application setup and Qt's picture work, in C++ (see `cpp/app.cpp`).

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("qview/cpp/app.h");

        fn qview_describe_application(version: &str);
        fn qview_has_window() -> bool;
        fn qview_suffixes() -> String;
        fn qview_image_info(path: &str) -> String;
        fn qview_copy_image(path: &str) -> bool;
        fn qview_render_markup(src: &str, marks: &str, dst: &str, clipboard: bool) -> String;
        fn qview_register_types();
        fn qview_set_icon_theme(name: &str);
    }
}
