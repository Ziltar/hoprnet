import type Hopr from '@hoprnet/hopr-core'
import type { LogStream } from '../logs'
import type { StateOps } from '../types'
import express from 'express'
import http from 'http'
import ws from 'ws'
import { debug } from '@hoprnet/hopr-utils'
import * as apiV2 from './v2'

const debugLog = debug('hoprd:api')

/**
 * Setup API servers & configuation
 * @param node HOPR instance
 * @param logs LogStream instance
 * @param stateOps HOPRd state
 * @param options
 */
export default function setupAPI(
  node: Hopr,
  logs: LogStream,
  stateOps: StateOps,
  options: {
    apiHost: string
    apiPort: number
    apiToken?: string
  }
): () => void {
  debugLog('Enabling Rest API v2 and WS API v2')
  const service = express()
  const server = http.createServer(service)

  apiV2.setupRestApi(service, '/api/v2', node, stateOps, options)
  apiV2.setupWsApi(server, new ws.Server({ noServer: true }), node, logs, options)

  return function listen() {
    server
      .listen(options.apiPort, options.apiHost, () => {
        logs.log(`API server on ${options.apiHost} listening on port ${options.apiPort}`)
      })
      .on('error', (err: any) => {
        logs.log(`Failed to start API server: ${err}`)

        // bail out, fail hard because we cannot proceed with the overall
        // startup
        throw err
      })
  }
}
