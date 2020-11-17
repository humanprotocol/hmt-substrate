const should = require('should')
const { createType } = require('@polkadot/types');

describe('kvStore tests', () => {
	const davePair = {address: "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy"}
	let initialBalanceAlice

	it("should set the values", async () => {
		
		const key = [1,2,3]
		const value = [4,5,6]
		const tx = await halva.polkadot.tx.kvStore.set(key, value)
		await passes(tx, 'set', alicePair)

		const storageValue = await halva.polkadot.query.kvStore.storage(alicePair.address, key)
		
		const expectedArr = halva.polkadot.createType('Vec<u8>', value)

		assert.deepEqual(storageValue, expectedArr, 'storage value should be set correctly')
	})
})