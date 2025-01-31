import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance } from '../../fixtures.js'
import { Balance, BalanceType } from '@hoprnet/hopr-utils'

let node = sinon.fake() as any

describe('GET /account/balances', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should get balance', async () => {
    const nativeBalance = new Balance('10', BalanceType.Native)
    const balance = new Balance('1', BalanceType.HOPR)
    node.getNativeBalance = sinon.fake.returns(nativeBalance)
    node.getBalance = sinon.fake.returns(balance)

    const res = await request(service).get('/api/v2/account/balances')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      native: nativeBalance.to_string(),
      hopr: balance.to_string()
    })
  })

  it('should return 422 when either of balances node calls fail', async () => {
    node.getBalance = sinon.fake.throws('')
    node.getNativeBalance = sinon.fake.returns(new Balance('10', BalanceType.HOPR))
    const res = await request(service).get('/api/v2/account/balances')
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec

    node.getBalance = sinon.fake.returns(new Balance('10', BalanceType.HOPR))
    node.getNativeBalance = sinon.fake.throws('')

    const res2 = await request(service).get('/api/v2/account/balances')
    expect(res2.status).to.equal(422)
    expect(res2).to.satisfyApiSpec
  })
})
