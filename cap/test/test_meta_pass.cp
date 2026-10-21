// cap/test/test_meta_pass.cp
// Comprehensive Phase 1 Test for MetaAnnotation

// 1. Definition with @target(class)
@target(class)
meta Controller(prefix: string);

// 2. Definition with @target(method)
@target(method)
meta Get(path: string = "");

// 3. Definition with @target(field)
@target(field)
meta Column(name: string);

// 4. Parameterless meta definition (without empty ())
meta JsonIgnore;

// 5. Unique meta definition with @once
@target(class) @once
meta Singleton;

// 6. Application with @reflect, positional args, multi-collection, and ad-hoc meta
@reflect
@Singleton
@Controller("/api/users")
@AdhocTag("v1.0")
class UserController {
    @JsonIgnore
    @Column("pwd_hash")
    secret: string;

    // Multiple annotations on the same method (automatic array aggregation without @Repeatable)
    @Get("/list")
    @Get("/all")
    fn list(): string {
        return "user_list";
    }

    // Default parameter call
    @Get()
    fn index(): string {
        return "index";
    }
}

fn main() {
    @print("--- MetaAnnotation Phase 1 Verification ---");
    let ctrl = new UserController();
    let r1 = ctrl.list();
    let r2 = ctrl.index();
    @print("Controller list: {s}", r1);
    @print("Controller index: {s}", r2);
    @print("🎉 MetaAnnotation Phase 1 verified successfully!");
}
