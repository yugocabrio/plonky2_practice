#[cfg(test)]
mod tests {
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
            proof::ProofWithPublicInputs,
        },
    };

    type F = GoldilocksField;

    // 2次の拡大体を使う
    // Fp[x]/ (x^2 - 7)
    const D: usize = 2;

    // hashにposeidonを利用して、proofを作る
    type C = PoseidonGoldilocksConfig;

    // keccakを使う場合
    // type C = KeccakGoldilocksConfig;

    #[test]
    fn test_plonky2_add() {
        // 回路のサイズや各種設定が入る構造体
        let config = CircuitConfig::standard_recursion_config();

        // 回路の制約を扱う
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // GoldilocksField上の1を定義する
        let one = F::from_canonical_u64(1);
        // あるいは
        // let one = F::ONE;

        let two = F::from_canonical_u64(2);

        // let three = F::from_canonical_u64(3);

        // 回路上の(空の)変数(ワイヤ)を定義する
        // まだ位置がわからないので、virtual targetとなる-> 最終的に、表の中の位置が固定され、Wireとなる
        let a = builder.add_virtual_target();
        let b = builder.add_virtual_target();

        // 制約a + b = cを課す
        let c = builder.add(a, b);

        // targetにwitness (値)を割り当てる
        // PartialWitnessは、target- witness間の関係を管理する構造体
        let mut pw = PartialWitness::new();

        // a = one, b = one
        pw.set_target(a, one);
        pw.set_target(b, one);

        // c = two
        pw.set_target(c, two);

        // circuitのbuildを行う
        let data = builder.build::<C>();

        // proofを生成する。仮に、invalidな条件の場合は、ここで失敗する
        let proof = data.prove(pw).unwrap();

        // proofのverify
        data.verify(proof).unwrap();
    }

    // a + b = cという制約が掛かっている
    struct InnerTarget {
        a: Target,
        b: Target,
        c: Target,
    }

    // inner circuit(再帰証明の対象になる回路)を生成する関数
    fn build_inner_circuit() -> (CircuitData<F, C, D>, InnerTarget) {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        let a = builder.add_virtual_target();
        let b = builder.add_virtual_target();
        let c = builder.add(a, b);

        let data = builder.build::<C>();
        let target = InnerTarget { a, b, c };
        (data, target)
    }

    fn generate_inner_proof(
        data: &CircuitData<F, C, D>,
        it: &InnerTarget,
    ) -> ProofWithPublicInputs<F, C, D> {
        let mut pw = PartialWitness::new();
        pw.set_target(it.a, F::ONE);
        pw.set_target(it.b, F::TWO);
        pw.set_target(it.c, F::from_canonical_u64(3));
        data.prove(pw).unwrap()
    }

    #[test]
    fn test_recursive_proof() {
        let (inner_data, inner_target) = build_inner_circuit();

        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // recursive proof
        let inner_verifier_data = builder.constant_verifier_data(&inner_data.verifier_only);
        let proof_with_pis = builder.add_virtual_proof_with_pis(&inner_data.common);

        // proof_with_pisというvirtual targetを検証する制約を回路に追加
        builder.verify_proof::<C>(&proof_with_pis, &inner_verifier_data, &inner_data.common);

        // inner proofの生成
        let inner_proof = generate_inner_proof(&inner_data, &inner_target);

        // witnessの割り当て
        let mut pw = PartialWitness::<F>::new();
        // proof_with_pisに値を割り当てる
        pw.set_proof_with_pis_target(&proof_with_pis, &inner_proof);

        // circuitを構築
        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();
        data.verify(proof).unwrap();
    }
}