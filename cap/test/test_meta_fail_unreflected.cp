// cap/test/test_meta_fail_unreflected.cp
// Negative test: verify compile error when calling reflect.type<T>() on a class with @meta but without @reflect

@meta(author: "AdHocMaster", timeout: 1200)
class UnreflectedMetaService {
    id: i32;
}

fn main() {
    let info = reflect.type<UnreflectedMetaService>();
}
