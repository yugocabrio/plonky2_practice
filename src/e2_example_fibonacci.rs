#[cfg(test)]
mod test {
    use std::array::IntoIter;

    use anyhow::Result;
    use plonky2::field::goldilocks_field::GoldilocksField;
    use plonky2::field::types::Field;
    use plonky2::iop::witness::{PartialWitness, WitnessWrite};
    use plonky2::plonk::circuit_builder::CircuitBuilder;
    use plonky2::plonk::circuit_data::CircuitConfig;
    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

    /// "I know the 100th element of the Fibonacci sequence, starting with constants a and b."
    /// When a == 0 and b == 1, this is proving knowledge of the 100th (standard) Fibonacci number.
    #[test]
    fn example_fibonaccici() -> Result<()> {
        // 拡大体
        const D: usize = 2;
        // ハッシュ関数(Poseidon)
        type C = PoseidonGoldilocksConfig;
        // 有限体
        type F = GoldilocksField;

        // 回路のサイズや各種設定が入る構造体
        let config = CircuitConfig::standard_recursion_config();
        // 回路の制約を扱う
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // the arithmetic circuit
        // ワイヤの構築
        let initial_a = builder.add_virtual_target();
        let initial_b = builder.add_virtual_target();

        let mut prev_target = initial_a;
        let mut cur_target = initial_b;

        for _ in 0..99 {
            // add 制約を課す
            let temp = builder.add(prev_target, cur_target);
            prev_target = cur_target;
            cur_target = temp;
        }

        // public inputs
        builder.register_public_input(initial_a);
        builder.register_public_input(initial_b);
        builder.register_public_input(cur_target);

        // witnessを割り当てる
        let mut pw = PartialWitness::new();
        pw.set_target(initial_a, F::ZERO);
        pw.set_target(initial_b, F::ONE);

        let data = builder.build::<C>();
        let proof = data.prove(pw)?;

        data.verify(proof)


    }


}