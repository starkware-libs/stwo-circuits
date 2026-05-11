use expect_test::{Expect, expect};

pub const BLAKE_G: Expect = expect![[r#"
    (1764683956 + 1445328965i) + (1747106891 + 897069792i)u
"#]];
pub const BLAKE_G_GATE: Expect = expect![[r#"
    (1821778611 + 324372422i) + (1838746257 + 2001370926i)u
"#]];
pub const BLAKE_GATE: Expect = expect![[r#"
"#]];
pub const BLAKE_OUTPUT: Expect = expect![[r#"
    (319146907 + 361207233i) + (824216992 + 1750935794i)u
"#]];
pub const BLAKE_ROUND_SIGMA: Expect = expect![[r#"
    (1097485784 + 777282969i) + (604469751 + 1994507613i)u
"#]];
pub const CIRCUIT_BLAKE_ROUND: Expect = expect![[r#"
"#]];
pub const M_31_TO_U_32: Expect = expect![[r#"
    (789166657 + 46563710i) + (663242899 + 1300432148i)u
"#]];
pub const QM_31_OPS: Expect = expect![[r#"
    (1788762958 + 1100891171i) + (538710207 + 1545850743i)u
"#]];
pub const RANGE_CHECK_15: Expect = expect![[r#"
    (2052111311 + 1192528295i) + (686829220 + 1363764450i)u
"#]];
pub const RANGE_CHECK_16: Expect = expect![[r#"
    (667904376 + 34949049i) + (420806061 + 1573330199i)u
"#]];
pub const TRIPLE_XOR: Expect = expect![[r#"
    (1371288620 + 713275058i) + (1771246253 + 534782712i)u
"#]];
pub const TRIPLE_XOR_32: Expect = expect![[r#"
    (1152376846 + 753053024i) + (1408555470 + 1523324023i)u
"#]];
pub const VERIFY_BITWISE_XOR_4: Expect = expect![[r#"
    (1408196330 + 613398876i) + (902685482 + 1598646629i)u
"#]];
pub const VERIFY_BITWISE_XOR_7: Expect = expect![[r#"
    (1148280325 + 1727535771i) + (231268374 + 1904995666i)u
"#]];
pub const VERIFY_BITWISE_XOR_8: Expect = expect![[r#"
    (2133776082 + 1230761515i) + (1058454893 + 1979176609i)u
"#]];
pub const VERIFY_BITWISE_XOR_9: Expect = expect![[r#"
    (841088250 + 1499053200i) + (1122584495 + 937798316i)u
"#]];
