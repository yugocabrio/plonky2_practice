#[cfg(test)]
mod test {
    use std::thread::Builder;

    use plonky2::{
        field::{goldilocks_field::GoldilocksField, types::Field},
        iop::{
            target::Target,
            witness::{PartialWitness, WitnessWrite},
        },
        plonk::{
            circuit_builder::CircuitBuilder,
            circuit_data::{CircuitConfig, CircuitData},
            config::PoseidonGoldilocksConfig,
            proof::{ProofWithPublicInputs, Proof},
        }
    };

    // 有限体 2^64-2^32+1
    type F = GoldilocksField;

    // 拡大体 F[x]/(x^"2"-7)
    const D: usize = 2;

    // Merkle Treeに使うHash関数, keccak256を使うとrecursionができない
    type C = PoseidonGoldilocksConfig;

    struct InnerTarget {
        a: Target,
        b: Target,
        c: Target,
    }

    // inner circuit(再帰証明)を生成する関数
    fn build_inner_circuit() -> (CircuitData<F, C, D>, InnerTarget) {
        // 回路のサイズや各種設定が入る構造体
        let config = CircuitConfig::standard_recursion_config();
        // 回路の制約を扱う(有限体と拡大体の引数入れる)
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // ワイヤの定義です
        let a = builder.add_virtual_target();
        let b = builder.add_virtual_target();
        // constraintの定義を行います
        let c = builder.add(a, b);

        let data = builder.build::<C>();
        let target = InnerTarget{a, b, c};
        (data, target)
    }

    fn generate_inner_proof(
        data: &CircuitData<F, C, D>,
        it: &InnerTarget,
    ) -> ProofWithPublicInputs<F, C, D> {
        // targetにwitnessを割りあてる
        // PartialWitnessは、targetとwitness間の関係を管理する構造体
        let mut pw = PartialWitness::new();
        pw.set_target(it.a, F::ONE);
        pw.set_target(it.b, F::TWO);
        pw.set_target(it.c, F::from_canonical_u64(3));
        data.prove(pw).unwrap()
    }

    #[test]
    fn practice_recursive_proof() {
        // 上に書いた build_inner_circuit 関数
        let (inner_data, inner_target) = build_inner_circuit();

        // recursive用のcircuit
        // 回路のサイズや各種設定が入る構造体
        let config = CircuitConfig::standard_recursion_config();
        // 回路の制約を扱う(有限体と拡大体の引数入れる)
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // recursive proof
        let inner_verifier_data = builder.constant_verifier_data(&inner_data.verifier_only);
        let proof_with_pis = builder.add_virtual_proof_with_pis(&inner_data.common);

        // proof_with_pisというvirtual targetを検証するconstraintを回路に追加
        builder.verify_proof::<C>(&proof_with_pis, &inner_verifier_data, &inner_data.common);

        // inner proofの生成
        let inner_proof = generate_inner_proof(&inner_data, &inner_target);

        // witnessの割り当て
        let mut pw = PartialWitness::<F>::new();
        // proof_with_pisを割り当てる
        pw.set_proof_with_pis_target(&proof_with_pis, &inner_proof);

        // circuitを構築
        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();
        data.verify(proof).unwrap();
    }

}