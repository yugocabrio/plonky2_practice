use std::hash::BuildHasher;

use anyhow::Result;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

/// "I know n * (n + 1) * ... * (n + 99)".
/// When n == 1, this is proving knowledge of 100!.
fn example_factorial() -> Result<()> {
    // 拡大体の設定
    const D: usize = 2;
    // FRIのマークルツリーで使うハッシュ関数
    type C = PoseidonGoldilocksConfig;
    // 有限体の設定
    type F = GoldilocksField;

    // 回路のサイズや各種設定が入る構造体
    let config = CircuitConfig::standard_recursion_config();
    // 回路の制約を扱う
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // the arithmetic circuit
    // ワイヤの定義（仮想上）
    // 初めの値
    let initial = builder.add_virtual_target();
    // 直近の値
    let mut cur_target = initial;

    for i in 2..101 {
        // 現在のループの回数をcircuitの定数に変換
        let i_target = builder.constant(F::from_canonical_u32(i));
        // 最近の値 = 最近の値 x ループ番目
        cur_target = builder.mul(cur_target, i_target);        
    }

    // public inputs are the initial value and result.
    builder.register_public_input(initial);
    builder.register_public_input(cur_target);

    // targetにwitnessを割り当てる
    let mut pw = PartialWitness::new();
    pw.set_target(initial, F::ONE);

    // circuitのbuild
    let data = builder.build::<C>();
    // proofの生成
    let proof = data.prove(pw)?;

    println!(
        "Factorial starting at {} is {}",
        proof.public_inputs[0], proof.public_inputs[1]
    );

    // proofのverify
    data.verify(proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_factorial() {
        assert!(example_factorial().is_ok());
    }
}
