impl T for S {
    default unsafe extern "C" fn f() {}
    default extern "C" fn g() {}
    default const unsafe fn h() {}
}
