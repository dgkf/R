(function() {
    var implementors = Object.fromEntries([["r",[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a> for <a class=\"enum\" href=\"r/object/enum.Obj.html\" title=\"enum r::object::Obj\">Obj</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a> for <a class=\"enum\" href=\"r/object/enum.Vector.html\" title=\"enum r::object::Vector\">Vector</a>"],["impl&lt;L, R, C, O, LNum, RNum&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;<a class=\"enum\" href=\"r/object/rep/enum.Rep.html\" title=\"enum r::object::rep::Rep\">Rep</a>&lt;R&gt;&gt; for <a class=\"enum\" href=\"r/object/rep/enum.Rep.html\" title=\"enum r::object::rep::Rep\">Rep</a>&lt;L&gt;<div class=\"where\">where\n    L: <a class=\"trait\" href=\"r/object/coercion/trait.AtomicMode.html\" title=\"trait r::object::coercion::AtomicMode\">AtomicMode</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"r/object/coercion/trait.MinimallyNumeric.html\" title=\"trait r::object::coercion::MinimallyNumeric\">MinimallyNumeric</a>&lt;As = LNum&gt; + <a class=\"trait\" href=\"r/object/coercion/trait.CoercibleInto.html\" title=\"trait r::object::coercion::CoercibleInto\">CoercibleInto</a>&lt;LNum&gt;,\n    R: <a class=\"trait\" href=\"r/object/coercion/trait.AtomicMode.html\" title=\"trait r::object::coercion::AtomicMode\">AtomicMode</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"r/object/coercion/trait.MinimallyNumeric.html\" title=\"trait r::object::coercion::MinimallyNumeric\">MinimallyNumeric</a>&lt;As = RNum&gt; + <a class=\"trait\" href=\"r/object/coercion/trait.CoercibleInto.html\" title=\"trait r::object::coercion::CoercibleInto\">CoercibleInto</a>&lt;RNum&gt;,\n    <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.82.0/std/primitive.tuple.html\">(LNum, RNum)</a>: <a class=\"trait\" href=\"r/object/coercion/trait.CommonNum.html\" title=\"trait r::object::coercion::CommonNum\">CommonNum</a>&lt;Common = C&gt;,\n    C: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;Output = O&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,\n    <a class=\"enum\" href=\"r/object/rep/enum.Rep.html\" title=\"enum r::object::rep::Rep\">Rep</a>&lt;C&gt;: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.82.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;O&gt;&gt;,\n    O: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,</div>"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a> for <a class=\"enum\" href=\"r/object/enum.OptionNA.html\" title=\"enum r::object::OptionNA\">OptionNA</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.82.0/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;Output = T&gt;,</div>"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[4266]}