fn foo() {
    builtin#offset_of(Foo, 0.1);
    builtin#offset_of(Foo, 0 .1.1.1);
    builtin#offset_of(Foo, 0. 1);
    builtin#offset_of(Foo, (0.1.bar.2));
}
