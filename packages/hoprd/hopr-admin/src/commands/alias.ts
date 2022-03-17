import type API from '../utils/api'
import PeerId from 'peer-id'
import { toPaddedString } from '../utils'
import { Command } from '../utils/command'

export default class Alias extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        default: [[], 'show aliases'],
        setAlias: [
          [
            ['hoprAddress', 'PeerId', true],
            ['string', 'Name', true]
          ],
          'set alias'
        ]
      },
      api,
      extra
    )
  }

  public name() {
    return 'alias'
  }

  public description() {
    return 'View aliases or alias an address with a more memorable name'
  }

  public async execute(log, query): Promise<void> {
    const [error, use, peerId, name] = this.assertUsage(query) as [string | undefined, string, PeerId, string]
    if (error) return log(error)

    // get latest known aliases
    const aliases = this.extra.getCachedAliases()

    if (use === 'default') {
      const names = Object.entries(aliases).map(([name]) => `${name} -> `)

      // no aliases found
      if (names.length === 0) {
        return log(`No aliases found.\nTo set an alias use, ${this.usage()}`)
      }

      const peerIds = Object.entries(aliases).map(([, alias]) => `${alias}`)
      return log(toPaddedString(names.map<[string, string]>((name, index) => [name, peerIds[index]])))
    } else {
      // sets aliases
      try {
        const response = await this.api.setAlias(peerId.toB58String(), name)

        if (response.status == 201) {
          return log(`Set alias '${name}' to '${peerId.toB58String()}'.`)
        } else {
          return log(`Failed to set alias with status code ${response.status}.`)
        }
      } catch (error: any) {
        return log(`Failed to set alias with unexpected error ${error.message}.`)
      }
    }
  }
}
