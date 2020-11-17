const should = require('should')

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

describe('hmToken negative tests', () => {
	it("should fail to transfer balances not enough", async () => {
		const amountToSend = "10000000000000000000000"
		try {
			const tx = halva.polkadot.tx.hmToken.transfer(charliePair.address, amountToSend)
			await passes(tx, 'Transfer', alicePair);
			should.fail("tx should have failed but didn't")
		} catch (e) {
			assert.equal(
				e.message,
				`Transfer : Failed with {"Module":{"index":10,"error":1}}`
			)
		}
	})
	it("should fail to transfer balances of 0", async () => {
		const amountToSend = "0"
		try {
			const tx = halva.polkadot.tx.hmToken.transfer(charliePair.address, amountToSend)
			await passes(tx, 'Transfer', alicePair);
			should.fail("tx should have failed but didn't")
		} catch (e) {
			assert.equal(
				e.message,
				`Transfer : Failed with {"Module":{"index":10,"error":0}}`
			)
		}
	})
	it("should fail to transfer bulk to many tos", async () => {
		const accounts = Array(101).fill(bobPair.address)
		const amounts = Array(101).fill("10")
		try {
			const tx = halva.polkadot.tx.hmToken.transferBulk(accounts, amounts, 11)
			await passes(tx, 'Bulktransfer', alicePair);
			should.fail("tx should have failed but didn't")
		} catch (e) {
			assert.equal(
				e.message,
				`Bulktransfer : Failed with {"Module":{"index":10,"error":4}}`
			)
		}
		
	})
	it("should fail to transfer bulk to much value", async () => {
		const accounts = Array(5).fill(bobPair.address)
		const amounts = Array(5).fill("1000000000000000000000000000000")
		try {
			const tx = halva.polkadot.tx.hmToken.transferBulk(accounts, amounts, 11)
			await passes(tx, 'Bulktransfer', alicePair);
			should.fail("tx should have failed but didn't")
		} catch (e) {
			assert.equal(
				e.message,
				`Bulktransfer : Failed with {"Module":{"index":10,"error":5}}`
			)
		}
	})
	it("should fail to transfer bulk mismatch accounts and values", async () => {
		const accounts = Array(5).fill(bobPair.address)
		const amounts = Array(6).fill("10")
		try {
			const tx = halva.polkadot.tx.hmToken.transferBulk(accounts, amounts, 11)
			await passes(tx, 'Bulktransfer', alicePair);
			should.fail("tx should have failed but didn't")
		} catch (e) {
			assert.equal(
				e.message,
				`Bulktransfer : Failed with {"Module":{"index":10,"error":3}}`
			)
		}

	})
	it("should fail silently and still transfer funds to one on a failing batch tx", async () => {
		const balanceAlice = await halva.polkadot.query.hmToken.balances(alicePair.address)
		const accounts = [bobPair.address, charliePair.address]
		const amounts = Array(2).fill(balanceAlice)
		const balanceOfCharlieBefore = await halva.polkadot.query.hmToken.balances(charliePair.address)
		const balanceOfBobBefore = await halva.polkadot.query.hmToken.balances(bobPair.address)
		const tx = halva.polkadot.tx.hmToken.transferBulk(accounts, amounts, 11)
		await passes(tx, 'Bulktransfer', alicePair);
		const balanceOfCharlieAfter = await halva.polkadot.query.hmToken.balances(charliePair.address)
		const balanceOfBobAfter = await halva.polkadot.query.hmToken.balances(bobPair.address)
		const calculatedBalanceofBob = Number(balanceOfBobBefore) + Number(balanceAlice)
		assert.equal(Number(balanceOfBobAfter.toString()), calculatedBalanceofBob.toString(), "bob should have gotten paid out")
		assert.equal(balanceOfCharlieBefore.toString(), balanceOfCharlieAfter.toString(), "charlie should not have gotten paid out")

	})

})
