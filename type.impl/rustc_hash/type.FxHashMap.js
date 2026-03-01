(function() {
    var type_impls = Object.fromEntries([["base_db",[]],["hir_ty",[]],["ide",[]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[14,14,11]}