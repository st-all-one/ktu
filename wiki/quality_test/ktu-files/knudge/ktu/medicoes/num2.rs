fn main() {
    let mut out = String::new();
    let xb = [1e-8_f64, 0.001, 0.5, 1.0, 10.0, 777.6666666666666, 1e6, 1e15];

    for x in xb {
        out.push_str(&format!("log1p {:>22} {:016x}\n", x, x.ln_1p().to_bits()));
    }
    for x in xb {
        out.push_str(&format!("log   {:>22} {:016x}\n", x, x.ln().to_bits()));
    }
    for x in xb {
        out.push_str(&format!("sqrt  {:>22} {:016x}\n", x, x.sqrt().to_bits()));
    }
    for x in xb {
        out.push_str(&format!("exp   {:>22} {:016x}\n", x.min(20.0), x.min(20.0).exp().to_bits()));
    }
    for (b, e) in [(1.2_f64, 3.7_f64), (0.5, 0.019178082191780823), (2.0, 0.5), (10.0, 0.3), (0.85, 30.0)] {
        out.push_str(&format!("pow   {:>6}^{:<22} {:016x}\n", b, e, b.powf(e).to_bits()));
    }

    // Formatação numero -> texto (o emissor TOON usa `{}`)
    let nums = [
        0.7, 1.0, 0.0, -0.0, 1e-7, 1e-6, 1.5e-5, 1e21, 1e20, 123456789012345680000.0, 0.1 + 0.2,
        3.0, 1e6, 1e-10,
    ];
    for n in nums {
        out.push_str(&format!("fmt   {:<26} => {}\n", format!("{:?}", n), n));
    }
    print!("{out}");
}
