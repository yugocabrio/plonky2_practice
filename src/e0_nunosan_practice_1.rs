#[cfg(test)]
mod test {
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
        }
    };

    // 有限体 2^64-2^32+1
    type F = GoldilocksField;

    // 拡大体 F[x]/(x^"2"-7)
    const D: usize = 2;

    // Merkle Treeに使うHash関数, keccak256を使うとrecursionができない
    type C = PoseidonGoldilocksConfig;

    #[test]
    fn practice_plonky2_add() {
        // 回路のサイズや各種設定が入る構造体
        let config = CircuitConfig::standard_recursion_config();
        // 回路の制約を扱う(有限体と拡大体の引数入れる)
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // GoldilocksField上の1を定義する
        let one = F::from_canonical_u64(1);
        // GoldilocksField上の2を定義する
        let two = F::from_canonical_u64(2);

        // ワイヤを定義する
        let a = builder.add_virtual_target();
        let b = builder.add_virtual_target();

        // constraint a+b=cを課す(定義したワイヤから入力に当てはまるものと、operation)
        let c = builder.add(a, b);

        // targetにwitnessを割りあてる
        // PartialWitnessは、targetとwitness間の関係を管理する構造体
        let mut pw = PartialWitness::new();

        // a = one, b = one
        pw.set_target(a, one);
        pw.set_target(b, one);

        // c = two
        pw.set_target(c, two);

        // circuitのbuildを行う
        let data = builder.build::<C>();

        // proofを生成する
        let proof = data.prove(pw).unwrap();

        // proofのverify
        data.verify(proof).unwrap();


    }


}    

