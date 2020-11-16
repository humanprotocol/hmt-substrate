
describe('hmToken positive tests', () => {
	const davePair = {address: "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy"}
	let initialBalanceAlice

	it("should verify balances on initialization", async () => {
		initialBalanceAlice = await halva.polkadot.query.hmToken.balances(alicePair.address)
		const initialBalanceBob = await halva.polkadot.query.hmToken.balances(bobPair.address)
		const totalSupply = await halva.polkadot.query.hmToken.totalSupply()

		assert.equal(totalSupply, '1000000000000000000000', 'total supply should be correct')
		assert.equal(initialBalanceAlice.toString(), totalSupply.toString(), "alice should have total supply")
		assert.equal(initialBalanceBob, "0", "bob should have no balance")
	}) 

	it("transfer tokens", async () => {
		const amountToSend = 1000000000000
		const totalSupply = await halva.polkadot.query.hmToken.totalSupply()
		const tx = await halva.polkadot.tx.hmToken.transfer(bobPair.address, amountToSend.toString())
		await passes(tx, 'transfer', alicePair)
		const balanceOfBobAfter = await halva.polkadot.query.hmToken.balances(bobPair.address)
		const balanceOfAliceAfter = await halva.polkadot.query.hmToken.balances(alicePair.address)
		const calculatedBalanceOfAlice = initialBalanceAlice - amountToSend
		const totalSupplyAfter =  await halva.polkadot.query.hmToken.totalSupply()

		assert.equal(balanceOfBobAfter, amountToSend, `bob should have recieved ${amountToSend} tokens`)
		assert.equal(balanceOfAliceAfter.toString(), calculatedBalanceOfAlice.toString(), `alice should have lost ${amountToSend} tokens`)
		assert.equal(totalSupply.toString(), totalSupplyAfter.toString(), 'total supply should not have moved')
	})

	it("transfers tokens in bulk", async () => {
		const amountToSend = 100
		const accounts = [charliePair.address, davePair.address]
		const amounts = [amountToSend, amountToSend]
		const totalSupply = await halva.polkadot.query.hmToken.totalSupply()
		const tx = await halva.polkadot.tx.hmToken.transferBulk(accounts, amounts, 1)
		await passes(tx, 'Bulktransfer', alicePair)
		const balanceOfCharlieAfter = await halva.polkadot.query.hmToken.balances(charliePair.address)
		const balanceOfDaveAfter = await halva.polkadot.query.hmToken.balances(davePair.address)
		const totalSupplyAfter =  await halva.polkadot.query.hmToken.totalSupply()

		assert.equal(balanceOfCharlieAfter, amountToSend, `charlie should have recieved ${amountToSend} tokens`)
		assert.equal(balanceOfDaveAfter, amountToSend, `dave should have recieved ${amountToSend} tokens`)
		assert.equal(totalSupply.toString(), totalSupplyAfter.toString(), 'total supply should not have moved')
	})
})
