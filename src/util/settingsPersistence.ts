import { shallowReactive } from 'vue'

// Each settings domain owns one channel; global and instance writes have separate queues.
export function createSettingsPersistence<Settings>(
  load: (directory: string | null) => Promise<Settings>,
  persist: (directory: string | null, settings: Settings) => Promise<Settings>,
) {
  const queues = new Map<string | null, Promise<unknown>>()
  const drafts = new Map<string | null, { settings: Settings }>()
  const errors = shallowReactive(new Map<string | null, string>())

  function stage(settings: Settings, directory: string | null = null) {
    drafts.set(directory, { settings: structuredClone(settings) })
    errors.delete(directory)
  }

  async function read(directory: string | null = null) {
    await queues.get(directory)
    const settings = drafts.get(directory)?.settings ?? (await load(directory))
    return {
      settings: structuredClone(drafts.get(directory)?.settings ?? settings),
      error: errors.get(directory) || '',
      dirty: drafts.has(directory),
    }
  }

  function save(directory: string | null = null) {
    const draft = drafts.get(directory)
    if (!draft) return Promise.resolve()
    const result = (queues.get(directory) || Promise.resolve())
      .then(() => persist(directory, structuredClone(draft.settings)))
      .then((saved) => {
        if (drafts.get(directory) === draft) {
          drafts.delete(directory)
          errors.delete(directory)
        }
        return saved
      })
      .catch((failure) => {
        if (drafts.get(directory) === draft) errors.set(directory, String(failure))
        throw failure
      })
    queues.set(
      directory,
      result.catch(() => {}),
    )
    return result
  }

  function write(settings: Settings, directory: string | null = null) {
    stage(settings, directory)
    return save(directory)
  }

  return { read, stage, save, write, errors }
}
