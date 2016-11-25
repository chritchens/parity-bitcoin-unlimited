use chain::Transaction;
use db::TransactionProvider;

pub fn transaction_fee(store: &TransactionProvider, transaction: &Transaction) -> u64 {
	let inputs_sum = transaction.inputs.iter()
		.fold(0, |accumulator, input| {
			let input_transaction = store.transaction(&input.previous_output.hash)
				.expect("transaction must be verified by caller");
			accumulator + input_transaction.outputs[input.previous_output.index as usize].value
		});
	let outputs_sum = transaction.outputs.iter()
		.fold(0, |accumulator, output| accumulator + output.value);
	inputs_sum.saturating_sub(outputs_sum)
}

pub fn transaction_fee_rate(store: &TransactionProvider, transaction: &Transaction) -> u64 {
	use ser::Serializable;

	transaction_fee(store, transaction) / transaction.serialized_size() as u64
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use db::{TestStorage, AsTransactionProvider};
	use test_data;
	use super::*;

	#[test]
	fn test_transaction_fee() {
		let b0 = test_data::block_builder().header().nonce(1).build()
			.transaction()
				.output().value(1_000_000).build()
				.build()
			.transaction()
				.output().value(2_000_000).build()
				.build()
			.build();
		let tx0 = b0.transactions[0].clone();
		let tx0_hash = tx0.hash();
		let tx1 = b0.transactions[1].clone();
		let tx1_hash = tx1.hash();
		let b1 = test_data::block_builder().header().nonce(2).build()
			.transaction()
				.input().hash(tx0_hash).index(0).build()
				.input().hash(tx1_hash).index(0).build()
				.output().value(2_500_000).build()
				.build()
			.build();
		let tx2 = b1.transactions[0].clone();

		let db = Arc::new(TestStorage::with_blocks(&vec![b0, b1]));

		assert_eq!(transaction_fee(db.as_transaction_provider(), &tx0), 0);
		assert_eq!(transaction_fee(db.as_transaction_provider(), &tx1), 0);
		assert_eq!(transaction_fee(db.as_transaction_provider(), &tx2), 500_000);

		assert_eq!(transaction_fee_rate(db.as_transaction_provider(), &tx0), 0);
		assert_eq!(transaction_fee_rate(db.as_transaction_provider(), &tx1), 0);
		assert_eq!(transaction_fee_rate(db.as_transaction_provider(), &tx2), 4_950);
	}
}
