/**
 * CP Reflection Standard Library (cap:reflect)
 * Zero-cost introspection & metadata reflection
 */

export class FieldInfo {
    name: string;
    typeName: string;
    meta: any;
}

export class MethodInfo {
    name: string;
    returnTypeName: string;
    paramNames: Array<string>;
    isAsync: bool;
    isStatic: bool;
    meta: any;
}

export class TypeInfo {
    name: string;
    isRef: bool;
    fields: Array<FieldInfo>;
    methods: Array<MethodInfo>;
    meta: any;
}
