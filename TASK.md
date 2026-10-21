## trait 与 类成员函数实例的所有权、借用与生命周期

很好，我们已经完成类似rust中的变量所有权、借用（只读、可变）、与生命周期。
现在我们继续完善trait 与 类成员函数实例的所有权、借用与生命周期。

Rust中的trait 类似Typescript中的Interface，但比Interface强大得多，借鉴 rust,实现/优化 cp 中的 trait, 除了struct，cp 语言有单继承class, trait 需支持 class 类。

trait 与 class 里的 self（类似Typescript this）实例在内部函数调用中，是否也涉及生命周期、可读借用、可变借用、所有权问题, cp的设计目标：既要实现类似rust的安全性，同时保持类似Typescript类似的语法简便性（比如使用缺省约定之类的方式，减少编写代码），给出你的完美设计方案

其中，我有几点建议, 供你参考、借鉴，如果你有更好方案，可以不采纳，并给出原因:

1. trait、class 成员函数可提供this（类似rust中的self）访问自身实例，成员函数互相调用省略this。
2. _ 符号等同于 this，测试样例直接使用_代替this，减少代码量，代码更简洁。
3. 模式匹配中使用 * 代替 _, 避免关键字冲突，模式匹配后续再完成。
4. trait与class中的函数访问 this, 是否需要类似rust那样，将this作为函数参数，用于标记借用、所有权？如不作为参数，如何解决借用、所有权。
5. 如作为第一个参数，缺省可读借用,可忽略。如需修改 this, 函数第一个参数,是否应填入 mut _ 或 move _, 表示可变借用或所有权转移？
6. 你是编程语言设计专家，给出你的设计方案。

## 函数返回类型（正常、错误、空），冒泡、捕获与模式匹配

我们继续完善、优化函数返回类型定义与错误冒泡。

函数返回类型涉及：正常、异常、null（空） 三种状态，借鉴rust与zig，同时保持类似Typescript的间接性，我提出如下设计建议，供你分析、参考、借鉴后，给出完美设计方案。

1. 函数返回类似Typescript使用符号`:`，返回void可省略，避免到处都是void。
2. 函数内部调用使用尾部 ? 错误向上冒泡。
3. 函数内部使用 catch 捕获错误（类似zig），但不使用try。
4. 使用 ?? “返回值”，简化遇到错误时直接返回值。
5. 为避免函数中调用到处出现尾部?，可使用 try {内部带错误返回类型调用忽略尾部?，编译时自动添加尾部?} catch err => {}，这里的try与zig、Typescript含义都不一样，只是一个省略尾部?区域标记，让代码更简洁、美观。
6. 函数返回类型 ?T!E，?可选空置，T正常，E错误集合，代码示例如下：

```ts
enum FileErr {
    FileNotFound,
    PermDenied,
    ReadEOF
}

fn load() : ?string ! FileErr {
    let txt = readFile("data.txt")?;
    // ?含义：如果res是Err，直接向上返回错误；成功则取出content绑定给txt
    return "prefix:" + txt; // 直接返回正常值，不用Ok()
}

// 返回：成功:string，控制返回null，错误:FileErr
fn readFile(path: string) : ?string ! FileErr {
    if (path == "") return Err(FileErr.FileNotFound);

    let str = fs.read(path);
    if (!str) return null
    return str; // ✅ 直接返回正常值，不用 Ok(...)
}

let res = readFile("data.txt");
// 模式匹配，使用 switch，代替rust的 match
let output = switch res {
    content => "读取成功：" + content, // ✅ 成功分支，直接绑定变量，不用Ok(content)
    null => "内容为空",
    Err(FileErr.FileNotFound) => "错误：文件不存在",
    Err(FileErr.PermDenied) => "错误：权限不足",
    Err(e) => "其他文件错误",
};
print(output);
```

?string: 表示返回可选，可能为 null（空）或字符串。
!E/!{FileNotFound, PermDenied}：表示错误集合。

7. 错误联合体语法 `T ! ErrType`；✅成功返回直接写原值，不用`Ok()`；错误返回必须写`Err(xxx)`，`Err` 是**专门用于错误联合体 `T ! ErrSet` 的语法标记**，不是普通结构体、不是类，是编译器识别的**关键字 / 内置构造器**，用来构造「错误域」的值。这样在switch模式匹配中，方便使用Err来区分错误分支，这种Err设计你觉得是否有必要？是否有更好建议。
8. ErrType集合借鉴了Rust的E和Zig的全局错误集，支持 ErrTye1 | ErrType2 多个集合合并。
9. ErrType支持匿名错误集合 {err1, err2}.
10. 匿名错误集合在赋值时对应使用匿名错误集合定义变量类型：let x:T!{err1, err2} = fn()，如编译期能自动推导，可省略。
11. 错误集，不采用zig的自动合并（多层嵌套调用错误集会膨胀），而采用rust的转换，转换语法避免rust的复杂性，需尽量简洁，减少手动编写代码。
12. 同时借鉴es6和Typescript，提供一个内部的统一Error对象（内置code、msg、cause等固定属性），在业务层，可将错误转换为统一Error对象处理，简化编码。

### 可选链

1. expr?.member：如果 expr为null，表达式返回null。
2. ?T：null可选类型，变量赋值时，使用自动推断，省略类型。
3. 链式调用：let city: ?string = user?.addr?.city，其中 city :string 类型定义可自动推导，可省略。
4. ?&T：可选只读借用，?&mut T：可选可变借用。
5. ?T ! ErrSet：失败返回ErrSet，成功返回 ?T。
6. null作为字面量，只能赋值给?T类型变量。

## Box胖指针

> 规范规则：底层库优先静态错误集合`{}` / enum；**只有动态加载插件、第三方用户智能体模块才选用`Box<dyn Trait>`动态错误**，避免不必要的运行时开销。

1. 底层开发，不使用Error，类似Rust 的动态胖指针，使用可向下转换（downcast）的错误实例，以获得错误详细信息，此时，cp需实现类似Rust的Box胖指针（数据指针、虚函数指针），用于向下转换，获得错误详细信息。
2. Box管理堆内存所有权，离开作用域自动Reease释放。
3. downcast()返回错误实例引用。

### 编译器实现要点

1. 当`impl Trait for Type`，编译器自动生成该类型对应的 vtable（不用像 Zig 手写虚表）；
2. `Box<dyn Trait>`自动生成胖指针；
3. `downcast<T>`内置方法，依赖类型 ID 做运行时类型校验；
4. 所有权规则：`Box`持有堆实例所有权，超出作用域自动 Drop 释放；
5. 借用规则沿用 CP 的借用 / 生命周期检查：胖指针同样受可读 / 可变借用互斥约束。

### 边界约束（重要，规避坑）

1. downcast 是**运行时检查**，编译期无法校验分支是否覆盖全部类型；
2. dyn trait 对象存在微小运行开销（虚表间接调用 + 堆分配），禁止在底层高性能核心路径默认使用；
3. 不允许`dyn Trait`放在栈上，必须由 Box 装箱（编译期报错）。
   4

### switch 模式匹配优化

1. 使用switch代替match，实现类似rust的模式匹配。
2. switch 是表达式，无穿透、无需`break`，支持模式守卫 `if`，支持：字面量、范围、元组、结构体、枚举、错误联合体解构、嵌套模式等。
3. 匹配错误联合体：不带`Err` = 成功分支；带`Err`= 错误分支；null = null分支。
4. `*` 代替 模式匹配中的 `_`。
5. 枚举 / 错误联合体开启编译期穷尽检查，`*`通配分支关闭穷尽校验

以上仅供你参考，需发挥你编程语言设计专家的技能，分析对比Rust、Zig与Typescript语言相关部分优缺点，实现cp语言该部分的完整设计方案。

## 宏编程、元编程、注解、装饰器、comptime

1. **Annotation（注解）**：只是**附加在程序元素上的元数据信息**。
   只存一段数据（键值对），**不执行代码，不修改类 / 方法**。
   作用：给编译器、运行时反射库读取标记信息。
2. **Decorator（装饰器）**：**可执行函数**，编译期 / 运行时函数，接收目标对象，可以修改类、覆盖方法、新增属性。装饰器内部可以读写元注解。
   装饰器**底层往往会写入 metadata**，但装饰器本身是一段逻辑。
   装饰器一般用于动态包装、修改类，AOP 横切逻辑等，如Nest.js。
3. metadata（元数据）：存储在类型上的静态键值信息，纯数据，不包含逻辑
4. **元注解（Meta Annotation）**：附加元数据标记，纯数据；编译期、运行时都可读，**不执行任何代码，不修改类、不注入方法**。类似 Java Annotation。用于路由、序列化标记、权限标记。
5. 反射：程序运行时，可以检查类、读取它有什么方法、读取附加的元标记。
   没有元数据，框架只能靠硬编码约定；有元数据，就可以做声明式开发：Nest、TypeORM 全靠这套。

这些编程语言的高级功能，我们可以使用“@”符号来定义。

### @符号使用示例：

1. 编译展开宏：@sprint(), 简化调用
2. 注解: @[] 类似rust的注解，与元注解有何差异？
3. 装饰器: @Controller("user")
   class UserCtrl{}
4. Zig comptime调用: let sz = sizeOf@(User)
5. **Meta 元注解 @Xxx (...)**：附加元数据标记，纯数据；编译期、运行时都可读，不修改类型，不注入方法。用于路由、序列化标记、权限标记。

# CP 语言：元注解 MetaAnnotation + 装饰器 Decorator 设计参考

> 前置依赖：CP Trait + Class 单继承规范；
> 定位：
>
> - **MetaAnnotation（元注解）**：纯静态标记，仅挂载键值元数据，**不执行代码，不修改类型**。编译期、运行时均可读取。对标 Java Annotation / Nest reflect-metadata，但语言原生内置。
> - **Decorator（装饰器）**：可执行转换函数，属于高级元编程，可修改类 / 结构体 / 方法；装饰器可以读写元注解。
>   分层学习：入门只学元注解；装饰器放在进阶章节，普通业务开发者可以长期不用。

> 文法符号约定
> `[ ... ]` 可选；`{ ... }` 重复 0 次或多次；`|` 多选；`(...)`分组；
> `Ident` 标识符；`Literal` 字面量（字符串 / 数字 / 布尔 /null）；`Type` CP 任意类型；`Block` 代码块；`Expr` 表达式。

---

## 第一部分：MetaAnnotation 元注解规范

### 1.1 语义规则

1. **元注解只存储元数据，不执行任何代码**
   `@Route("/user")` 等价于向目标元素写入元数据记录：`Route = "/user"`。
   无函数调用、不修改目标类型结构、不新增 / 覆盖方法。
2. **元数据存储模型：命名空间键值对**
   每个被注解的元素（class /method/field /param/trait）持有独立元数据表：

```
@ApiInfo(title="用户接口", version="1.0")
class UserController {}
```

存入元数据：`ApiInfo = {title:"用户接口", version:"1.0"}`

- key：注解标识符（可选自动增加包命名空间，防止冲突）
- value：支持字面量、简单对象、数组。

3. **编译期 + 运行时双可读（核心优势，区别于 TS reflect-metadata）**

```
// CP 内置反射API（运行时）
reflect.getMeta(target: Type | Function | Instance, key: string): any?
reflect.setMeta(target: Type | Function | Instance, key: string, value: any): void
reflect.getMetaKeys(target): Array<string>
```

- **编译期**：编译器、宏、静态检查器可以读取元注解，做校验、代码生成。
- **运行时**：内置反射 API 读取元数据。

4. **多个同 key 元注解：默认覆盖；支持标记 `@Repeatable` 元注解允许数组收集**

```
@Repeatable
@Get("/list")
@Get("/all")
class UserController{}
```

`@Repeatable` 是 CP 内置基础元注解，标记后，多次使用同一个注解会收集成数组，而不是覆盖。

5. **元注解作用目标限制**

可以定义元注解，限制它只能附着在 class /method/field 等特定元素上。

```
@Target(Class)
annotation Route(path: string);
```

> `annotation` 关键字：用来**自定义元注解定义**（CP 新增关键字）

```
AnnotationDef ::= MetaAnnotationList "annotation" Ident "(" [ParamList] ")" ";"
```

示例：

```
@Target(Class)
annotation Route(path: string);

@Target(Method) @Repeatable
annotation Get(path: string);
```

6. **元注解不能带业务逻辑，参数仅支持编译期常量字面量**
   不允许在元注解参数写运行时表达式，保证编译期可读取。

## 1.2 元注解使用示例

```
// 自定义元注解定义
@Target(Class)
annotation Controller(prefix: string);

@Target(Method) @Repeatable
annotation Get(path: string);

// 使用元注解
@Controller("/user")
class UserController {
    @Get("/list")
    fn getUserList(): Array<User> {
        return [];
    }

    @Get("/detail")
    fn getUserDetail() {
    }
}

// 运行时读取元数据
fn scanRoutes(ctrlType: Type) {
    var prefix = reflect.getMeta(ctrlType, "Controller");
}
```

## 1.3 错误规范

- 错误：`@Get` 只能标记方法，不能标记类。

> 元注解`Get`的 Target 限制仅允许附着在方法上，当前放置在类定义处。
> 修复：移动注解到方法。

---

## 第二部分：Decorator 装饰器设计参考

> 核心定位：装饰器 = 编译期执行的转换函数，**可以修改目标类型 / 方法**；装饰器内部可以读写元注解。
> 区分：元注解是**数据标记**；装饰器是**转换函数**。

## 2.1 语义规则

1. **执行时机：编译期执行（CP 默认设计）**
   装饰器函数**在编译阶段运行**，接收目标描述对象，返回修改后的定义。

> 和 TS 不同：TS 装饰器运行时执行；CP 默认编译期执行，减少运行开销。
> 可选扩展：增加 `@runtime` 标记，支持运行时装饰器（AOP 场景）。

2. **装饰器入参：目标元描述对象**
   装饰器函数接收一个描述对象，包含目标信息：
   - 类型种类（class /struct/method）
   - 成员列表
   - 现有元注解集合
   - 可以增删方法、修改字段、读写元数据
3. **装饰器能力边界**
   ✅ 新增 / 替换方法
   ✅ 修改方法实现，包装函数（AOP 拦截）
   ✅ 读取、添加、删除目标上的元注解
   ✅ 添加字段（类支持；struct 添加字段需要遵守内存布局规则）
   ❌ **不能修改父类继承链（extends）**，防止破坏单继承体系。
   ❌ 不能修改 trait 契约实现清单（implements 列表禁止在装饰器中修改）。

> 这条约束非常关键：**装饰器不允许动态修改继承与 trait 实现关系**。类型系统的契约必须源码显式声明，保证类型检查可靠。

4. **堆叠顺序：自上而下执行**

```
@DecA()
@DecB()
class X {}
```

执行顺序：先执行`DecB`，再执行`DecA`（和 TS 保持一致，符合开发者习惯）。5. **装饰器与元注解共存**
元注解先解析存入元数据表，**然后装饰器执行**。装饰器可以读取已经解析好的元注解，也可以新增元注解。

```
@MetaTag("abc") // 元注解，先写入meta
@MyDecorator()  // 装饰器，编译期执行，可以读取MetaTag
class Demo{}
```

6. **装饰器不能替代 trait**
   装饰器添加的方法：**类型系统可见性受限制**。

> 装饰器编译期追加的方法，类型系统可以识别，但**不能作为泛型约束**。
> `T: SomeDecorated` 非法。能力契约仍然只能使用 trait。

## 2.2 装饰器定义 & 使用示例

```
// 定义装饰器：日志装饰器
decorator LogMethod() {
    // target: 目标元素描述对象（编译期元对象）
    return fn(target) {
        // 读取原有元注解
        var name = target.name;
        var oldImpl = target.func;
        // 包装原方法，注入日志逻辑
        target.func = fn(args) {
            print("进入方法：" + name);
            var res = oldImpl(args);
            print("退出方法：" + name);
            return res;
        }
        // 装饰器新增元注解
        reflect.setMeta(target, "Log:enabled", true);
        return target;
    }
}

// 使用装饰器 + 元注解混合
@Permission("admin") // 元注解，仅存数据
@LogMethod()         // 装饰器，编译期包装函数
fn deleteUser(id: int) {
    print("删除用户 " + id);
}
```

## 2.3 错误规范示例

```
错误：装饰器MyDec尝试修改类的implements trait列表，该操作不被允许。
类型实现的trait必须在源码显式声明，不能由装饰器动态修改。
```

```
错误：装饰器不能修改class的extends父类。
单继承链在源码固定，装饰器不允许修改继承关系。
```

## 第三部分：反射系统规范（Reflection）

> 前置依赖：CP Class 单继承、Trait、元注解 MetaAnnotation、装饰器 Decorator 规范
> 说明：
>
> - **反射系统**：提供编译期 & 运行时统一的类型内省能力，读取类型结构、成员、元注解；
> - **宏系统**：编译期执行的代码生成能力，可读取元注解、类型信息，输出 CP 源码 AST；宏**不参与运行时**，属于编译期元编程。
>   文法符号约定
>   `[ ... ]` 可选；`{ ... }` 重复 0 次或多次；`|` 多选；`(...)`分组；
>   `Ident` 标识符；`Literal` 字面量；`Type` CP 任意类型；`Block` 代码块；`Expr` 表达式；`AST` 抽象语法树节点。

## 3.1 设计目标

1. 统一两套反射上下文：**编译期反射上下文（供宏、编译器校验）**、**运行时反射上下文（程序运行时 API）**。
2. 读取能力：类 / 结构体 / Trait / 方法 / 字段 / 参数信息、元注解数据。
3. 约束：**反射只能读，不能修改类型结构**。

> 运行时不能新增字段、不能修改继承链、不能修改 implements trait 列表；
> 只有装饰器（编译期）可以修改类型定义，运行时反射只读。

4. 类型安全：反射 API 返回带类型的描述对象，减少`any`；关闭反射可做裁剪（发布选项：关闭运行时反射，减少二进制体积）。

## 3.2 核心类型模型（反射对象模型）

### 顶层反射类型

```
// TypeKind：区分类型种类
enum TypeKind {
    Class,
    Struct,
    Trait,
    Primitive, // int / string / bool
    Array,
    GenericParam,
    DynTrait
}

// 类型描述对象
struct TypeInfo {
    kind: TypeKind;
    name: string;
    fullName: string; // 带包名，如 std::string
    // 继承信息（仅Class有效）
    superType: TypeInfo?;
    // 实现的trait列表
    implements: Array<TypeInfo>;
    // 字段列表
    fields: Array<FieldInfo>;
    // 方法列表
    methods: Array<MethodInfo>;
    // 元注解
    meta: MetaCollection;
}

// 字段信息
struct FieldInfo {
    name: string;
    type: TypeInfo;
    isMut: bool;
    meta: MetaCollection;
}

// 方法信息
struct MethodInfo {
    name: string;
    params: Array<ParamInfo>;
    returnType: TypeInfo?;
    isStatic: bool;
    meta: MetaCollection;
}

// 参数信息
struct ParamInfo {
    name: string;
    type: TypeInfo;
    isMut: bool;
    meta: MetaCollection;
}

// 元注解集合（MetaAnnotation存储容器）
struct MetaCollection {
    fn get(key: string): any?;
    fn getAll(key: string): Array<any>; // 读取@Repeatable注解数组
    fn keys(): Array<string>;
}
```

## 3.3 内置 `reflect` 模块 API

> API 在**编译期上下文**和**运行时上下文**均可调用，但能力有差异：
>
> - 编译期：完整读取全部 TypeInfo；
> - 运行时：仅保留开启了反射标记的类型。

```
module reflect {
    // 获取类型信息
    fn typeInfo<T>(): TypeInfo;
    fn typeInfoOf(value: any): TypeInfo;

    // 元数据快捷API（封装MetaCollection）
    fn getMeta(target: TypeInfo | MethodInfo | FieldInfo, key: string): any?;
    fn getMetaAll(target: TypeInfo | MethodInfo | FieldInfo, key: string): Array<any>;
    fn getMetaKeys(target: TypeInfo | MethodInfo | FieldInfo): Array<string>;

    // 实例相关
    fn getInstanceType(instance: any): TypeInfo;
    fn newInstance(type: TypeInfo, args: Array<any>): any;
}
```

### 启用运行时反射的元注解

```
// 内置元注解：标记类型开启运行时反射信息
annotation ReflectEnable();
```

示例：

```
@ReflectEnable
class User {
    id: int;
    name: string;
}
```

> 不加`@ReflectEnable`：运行时**不存在该类型的 TypeInfo**，节约程序体积；编译期宏仍然可以读取。

## 3.4 使用示例

```
@ReflectEnable
@Controller("/user")
class UserController {
    @Get("/list")
    fn list(): Array<User> {
        return [];
    }
}

fn main() {
    // 获取类型信息
    var type = reflect.typeInfo<UserController>();
    // 读取元注解
    var routePrefix = reflect.getMeta(type, "Controller");
    print(routePrefix); // /user

    // 遍历方法读取Get元注解
    for method in type.methods {
        var path = reflect.getMeta(method, "Get");
        if path != null {
            print("路由：" + path);
        }
    }
}
```

## 3.5 语义约束

1. **运行时反射只读**：`reflect` API 没有`setMeta`；运行时不能修改类型结构。

> 元注解写入仅在编译期完成。

2. 编译期反射：**全部类型默认开放**，不受`@ReflectEnable`限制。宏在编译阶段可以访问所有类型。
3. 泛型类型：每个实例化后的泛型类型，独立生成 TypeInfo。
4. 安全限制：私有字段 / 方法，运行时反射默认不可读取，需要额外`@ReflectPrivate`注解开启。

## 3.6 编译报错示例

```
错误：类型 User 未标记 @ReflectEnable，运行时无法获取其类型信息。
修复：在 class/struct 上添加 @ReflectEnable，或者在编译期宏中读取。
```

---

## 第四部分：CP 编译期宏系统规范（Macro）

> 定位：**编译期宏，执行时机：AST 解析完成、类型检查之前 / 可选类型检查后**。
> 宏是编译期函数，输入 AST 节点，输出新 AST 节点；**宏代码只在编译器内部执行，不会编译进最终二进制**。
> 宏可以调用**编译期版本的 reflect API**，读取类型、元注解，自动生成代码。
> 区分：
>
> - 元注解：纯标记数据；
> - 装饰器：编译期转换单个类型 / 方法；
> - 宏：批量生成、新增代码文件 / 类型 / 函数，能力比装饰器更强。

## 4.1 语义规则

1. **执行时机**
   宏执行在编译阶段，**不会编译到最终程序**。
   执行顺序：
   源码解析 AST → 元注解收集 → 宏展开执行（可调用编译期 reflect） → AST 替换 / 新增节点 → 装饰器执行 → 类型检查。

> 可选模式：类型检查后宏（type-aware macro，高级），可以读取类型检查结果。

2. **宏的输入输出：AST 节点**
   宏函数参数是 AST 节点；返回值是 AST 节点（可以返回单个节点、节点数组）。
   CP 内置 AST 构造 API，用来生成类、函数、字段等语法节点。
3. **宏能力范围**
   ✅ 读取目标元素的 TypeInfo、元注解（编译期 reflect）
   ✅ 新增函数、结构体、类、trait 实现代码
   ✅ 批量生成 impl 块（非常适合序列化、ORM 场景）
   ✅ 条件生成代码（if 分支）
   ❌ 不能修改已经存在的 AST 节点（只能新增、替换整个节点，不原地修改）
   ❌ 不能修改源码里已经写好的 extends /implements 声明
   ❌ 不能执行 IO、网络（编译期沙箱限制，仅纯计算）
4. **宏参数**
   宏参数支持字面量、标识符、AST 节点；参数全部是编译期常量。
5. **宏与装饰器的边界**

- **装饰器**：只能转换**当前这一个元素**（例如修饰单个 class），不能新增顶层其他类型；
- **宏**：可以一次性生成大量独立的顶层代码，可以跨元素生成。

> 举例：
>
> - 装饰器：给当前 class 的方法做日志包装；
> - 宏：扫描所有带`@Table`元注解的 class，批量生成数据库 CRUD 全部代码。

## 4.2 内置 AST 构造 API（宏内可用）

```
// 宏内置 ast 模块，用来生成语法树
module ast {
    // 创建类
    fn class(name: string, extends: TypeInfo?, implements: Array<TypeInfo>, members: Array<AstNode>): AstNode;
    // 创建函数
    fn fnDef(name: string, params: Array<ParamInfo>, retType: TypeInfo?, body: AstNode): AstNode;
    // 创建字段
    fn field(name: string, ty: TypeInfo, init: AstNode?): AstNode;
    // 创建impl块
    fn impl(traitTy: TypeInfo, targetTy: TypeInfo, methods: Array<AstNode>): AstNode;
    // 字面量节点
    fn lit(value: Literal): AstNode;
}
```

## 4.3 宏示例：序列化宏

```
// 定义元注解，标记数据库表
@Target(Class)
annotation Table(name: string);

// 定义宏
macro GenSerializer() {
    return fn(targetAst) {
        // 编译期反射读取目标类信息
        var tyInfo = reflect.typeInfo(targetAst);
        var className = tyInfo.name;
        var fields = tyInfo.fields;

        // 构建toJson方法代码AST
        var methodBody = ast.block([
            // ... 循环拼接字段序列化逻辑
        ]);

        var serializeMethod = ast.fnDef(
            "toJson",
            [ast.param("self", ast.typeRef(className))],
            ast.typeRef("string"),
            methodBody
        );

        // 生成 impl JsonSerializable for XXX
        var implBlock = ast.impl(
            ast.typeRef("JsonSerializable"),
            ast.typeRef(className),
            [serializeMethod]
        );
        // 返回新增的impl AST节点，编译器追加到源码
        return [targetAst, implBlock];
    }
}

// 使用宏
@GenSerializer()
@Table("user")
class User {
    id: int;
    name: string;
}
```

> 宏执行后，自动生成：
>
> ```
> impl JsonSerializable for User {
>     fn toJson(self): string {
>         return `{"id":${self.id},"name":"${self.name}"}`;
>     }
> }
> ```

## 4.4 宏沙箱与安全规则

1. 宏运行在编译器沙箱，禁止文件读写、网络访问、系统调用。
2. 宏展开后的 AST，全部要经过正常类型检查，宏不能绕过类型系统。
3. 循环宏展开检测：防止无限递归宏生成代码，编译期检测并报错。

## 4.5 报错示例

```
错误：宏GenSerializer产生无限递归展开。
宏展开过程不断生成新宏调用，编译终止。
```

```
错误：宏尝试修改类 User 的 extends 父类，该操作不允许。
宏只能新增代码，不能修改已声明的继承关系。
```

# CP 语言：CT Runtime（编译期运行时）+ const fn 规范 + 编译循环依赖规避方案

> 前置依赖：CP 元注解、装饰器、宏、反射 TypeInfo 规范
> 文档分为两大块：
> ① CT Runtime + const fn 完整规范
> ② 宏 / 编译期元编程的循环依赖问题、检测机制、规避方案

---

# 第五部分：CT Runtime + const fn 设计参考

## 1. 设计目标

1. 提供**沙箱隔离的编译期执行虚拟机**，允许一部分 CP 代码在编译阶段运行。
2. 区分两套执行上下文：**运行时（RT）**（程序最终二进制运行）、**编译期（CT）**（编译器内部执行）。
3. `const fn`：标记为编译期可执行的普通函数；可在编译期求值，也可在运行时正常调用（双态函数）。
4. 权限隔离：普通`const fn`仅允许纯计算、编译期反射读操作；**只有 macro 函数额外开放 AST 修改 API**。
5. 无副作用：CT 上下文中默认禁止 IO、文件、网络、随机数，保证编译可复现。

## 2. 语义规则

### 2.1 两套上下文区分

1. **RT（运行时上下文）**
   最终程序二进制执行；所有普通变量、普通函数。
2. **CT（编译期上下文 / CT Runtime 沙箱）**
   编译器内部执行；`const fn`在常量表达式、宏内部会进入 CT 上下文执行。

> 同一个`const fn`，**可以跨上下文**：
>
> - 编译期：在 const 表达式、宏内部 → CT Runtime 执行
> - 运行时：普通代码调用 → 正常机器码执行

### 2.2 const fn 约束（编译期执行时）

✅ 允许：

- 基础运算：加减乘除、比较、逻辑运算
- 分支`if`、循环`for/while`
- 调用其他`const fn`
- 编译期只读反射 API：`reflect.typeInfo()`、`reflect.getMeta()`等（只读，不能修改类型）
- 字面量、ct 结构体实例创建

❌ CT 上下文中禁止：

- 分配堆内存（`alloc`）、所有权 move / 借用相关运行时内存操作
- IO、文件读写、网络、系统调用
- 随机数、时间获取（编译时时间每次编译不一样，破坏可复现）
- 修改全局状态
- AST 操作 API（`ast`模块，**仅 macro 可以导入使用**，普通 const fn 无权访问）

> 核心权限隔离：
> `ast`模块 = 高级特权 API，仅 macro 函数的 CT 上下文可见。普通 const fn 就算在 CT 运行，`use ast;`直接编译报错。

### 2.3 `ct struct` 编译期专属结构体

`ct struct`：**只存在编译期，不会生成运行时内存布局，不会打包进最终二进制**。
用于宏 /const fn 内部组织编译期计算数据，例如临时存储字段列表、元注解信息。

```
ct struct FieldMeta {
    name: string;
    tyName: string;
}
```

> 运行时代码中不能实例化`ct struct`。

### 2.4 常量求值规则

```
const fn sum(a:int, b:int) -> int {
    return a + b;
}

// 编译期直接求值，编译后直接替换为字面量 30
const TOTAL: int = sum(10, 20);
```

- 当`const fn`入参全部是常量表达式 → **编译期执行求值**
- 如果入参包含运行时变量 → 编译期不展开，生成普通函数调用，运行时执行

### 2.5 CT Runtime 内置标准库子集

CT 环境内置裁剪后的标准库，只包含纯无副作用函数：

- `std::string`：字符串拼接、查找、截取
- `std::array`：数组遍历、map/filter、长度获取
- `reflect`：编译期只读反射 API（TypeInfo、元注解读取）

> 不包含：std::fs、std::net、std::rand。

### 2.6 错误诊断示例

```
错误：编译期const fn 调用了禁止的系统调用 std::fs::read。
CT Runtime沙箱不允许IO操作，保证编译结果可复现。
```

```
错误：普通const fn尝试导入ast模块。
ast AST构造API仅macro函数可访问。
```

## 代码示例

```
// CT专属结构体，仅编译期存在
ct struct FieldSummary {
    name: string;
    typeName: string;
}

// const fn：编译期可执行，双态函数
const fn collectFields(info: TypeInfo) -> Array<FieldSummary> {
    var res = Array<FieldSummary>{};
    for f in info.fields {
        res.push(FieldSummary{
            name: f.name,
            typeName: f.type.name
        });
    }
    return res;
}

// 宏：运行在CT Runtime，拥有ast特权API
macro GenJson() {
    return fn(targetAst) {
        var ty = reflect.typeInfo(targetAst);
        var fieldList = collectFields(ty); // 调用const fn编译期函数
        // 利用ast模块生成impl代码
        var implNode = ast.impl(...);
        return [targetAst, implNode];
    }
}

// 使用
@GenJson()
class User {
    id: int;
    name: string;
}
```

---

# 第六部分：编译循环依赖问题 + 检测与规避方案

> 核心风险来源：CP 支持**类型检查之后执行宏**，宏可以新增 AST，新增 AST 又需要类型检查，新 AST 又可能触发宏展开，形成循环。
> Rust 通过【宏全部在类型检查前执行】，牺牲能力来规避这个循环；CP 选择保留强能力，**用静态规则 + 循环检测 + 分层展开策略解决循环依赖**。

## 1. 循环依赖的两种场景

### 场景 A：宏生成代码，生成的代码上又附加宏，触发无限展开

```
// 伪代码 危险循环
@MacroA()
class A {}

// MacroA展开后自动生成class B，并且给B加上@MacroA()
// 编译器展开A → 生成B，B又触发MacroA → 生成C，无限递归
```

### 场景 B：类型互相引用，宏读取对方 TypeInfo，形成类型检查循环

```
@GenMeta()
class A {
    b: B;
}
@GenMeta()
class B {
    a: A;
}
```

类型检查 A，需要 B 的 TypeInfo；检查 B，需要 A 的 TypeInfo；而`@GenMeta`宏在类型检查后执行，读取 TypeInfo，互相等待，锁死编译管线。

## 2. 分层宏展开时序（核心解决方案）

把宏分成**两个阶段**，严格划分执行时机，禁止跨阶段乱序：

> Phase 1：**类型检查前宏（Pre-TypeCheck Macro）**
> 和 Rust 过程宏一样，在 AST 解析完成、类型检查之前执行。
> 限制：**不能读取完整 TypeInfo**，只能读取原始 AST 与元注解。
> 适合：简单语法转换、轻量装饰器。
> ✅ 不会引入循环风险，因为此时类型推导还没开始。

> Phase 2：**类型检查后宏（Post-TypeCheck Macro，CP 独有高阶宏）**
> 全部类型完成一轮基础类型检查之后，才执行。
> 限制：
>
> 1. Phase2 宏**允许读取完整 TypeInfo**；
> 2. Phase2 宏**只允许新增 impl 块，不允许新增 class/struct/trait 顶层类型**。
>
> > 关键约束！Phase2 宏**不能新增新的类 / 结构体**，只能新增 impl 实现。
> > 这一条直接切断场景 A 无限递归根源：Phase2 宏无法生成带宏注解的新顶层类型。

> 整体编译管线更新：
> 源码解析 → 收集元注解 → Phase1 宏展开 → 第一轮类型检查 → Phase2 宏展开（仅新增 impl） → 对新增 impl 做局部类型校验 → 借用检查 → 生成 IR

> 为什么这个约束有效：
> impl 块只是给已有类型追加方法，**不会创建新的类型定义**，不会产生新的宏调用，不会触发新一轮顶层宏展开。
> ORM、序列化生成全部只需要 impl，完全满足业务元编程需求。

## 3. 循环依赖检测机制

### 3.1 宏展开栈追踪

编译器维护**宏展开调用栈**，每一次进入宏展开，压栈；展开完成出栈。

- 如果准备执行的宏已经在栈中 → 判定为循环展开，直接终止编译，给出诊断。
  示例报错：

```
错误：宏GenJson发生循环展开。
展开栈：User -> GenJson -> Order -> GenJson
提示：宏展开过程中递归触发自身，不允许。
```

### 3.2 类型循环引用检测（类型图）

编译器维护一张**类型依赖有向图**。
当宏读取`TypeInfo`时，记录依赖边 `当前类型 → 被引用类型`。
如果新增边构成环，编译器报错。

> 区分：
> ✅ 允许**语法层面的循环类型**（A 包含 B，B 包含 A，使用 Box / 堆指针，Rust 也支持）
> ❌ 禁止**宏在类型检查阶段循环读取 TypeInfo 导致的编译死锁**。

### 3.3 增量标记：宏展开结果缓存

对同一个宏 + 同一个目标 AST，缓存展开产物；重复遇到直接复用缓存，不重复执行宏。
缓存 key：目标节点哈希 + 宏参数哈希。
用于增量编译，也避免重复执行宏带来的循环。

## 4. 源码层面的语言规则，引导用户规避循环

1. **Phase2（类型检查后）宏禁止生成新 class/struct/trait**，仅允许 impl、函数。
2. Phase1 宏可以新增顶层类型，但 Phase1 宏**不能访问完整 TypeInfo**，只能操作原始 AST。

> 取舍：
>
> - 想要最强能力（读取完整字段类型）→ Phase2，只能生成 impl
> - 想要生成新类 / 结构体 → Phase1，但拿不到完整 TypeInfo
>   开发者二选一，从根源切断循环。

3. 宏内禁止动态给生成的节点追加宏注解。

> 宏展开生成的 AST 节点，不允许附加`@MacroX`。只有手写源码的 AST 节点，才允许挂载宏调用。
> 这个规则杜绝 “宏生成代码，生成代码再触发宏” 的无限递归。

## 5. 循环依赖报错提示模板

```
错误：类型A与类型B形成编译期元信息循环依赖。
@GenJson 宏读取A的类型信息，依赖B；读取B的类型信息依赖A。
建议：
1. 将其中一个类型的字段改为堆分配（Box<T>）打破类型图强依赖；
2. 或者改用Phase1宏，不依赖完整TypeInfo。
```

## 6. 边界总结

表格

| 项目                    | Phase1 宏（类型检查前）            | Phase2 宏（类型检查后，CP 高阶）               |
| ----------------------- | ---------------------------------- | ---------------------------------------------- |
| 执行时机                | AST 解析完成，类型检查之前         | 第一轮基础类型检查完成之后                     |
| 是否能读取完整 TypeInfo | ❌ 只能原始 AST、元注解            | ✅ 完整 TypeInfo、泛型实例化信息               |
| 允许生成什么            | class / struct / trait / impl / fn | 仅允许 impl 块、辅助函数，**禁止新增顶层类型** |
| 循环风险                | 低，但需要自己控制                 | 中等；靠规则限制只能生成 impl，阻断无限递归    |
| 适用场景                | 语法转换、生成新类型               | 序列化、ORM，基于完整字段信息生成 impl         |

---

# 整体汇总（CT + const fn + 循环约束）

1. CT Runtime 是沙箱虚拟机，提供编译期执行 CP 代码的底座；`const fn`是双态函数，可编译期求值，也可运行时执行。
2. 权限隔离：普通 const fn 只能做计算、只读反射；只有 macro 才有 AST 构造 API。
3. 宏拆成两个阶段：Phase1（类型检查前，可新建类型，无完整 TypeInfo）；Phase2（类型检查后，拥有完整 TypeInfo，但只能生成 impl，不能新建类型）。
4. 编译器维护**宏展开栈 + 类型依赖图**，自动检测循环，及时报错。
5. 禁止宏生成的 AST 节点挂载宏注解，从根源杜绝无限宏展开。
