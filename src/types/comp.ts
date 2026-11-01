export type CompSource = 'modrinth' | 'curseforge'
export type CompSourceFilter = 'all' | CompSource

// Shared resource model used by both providers, search, favorites and downloads.
export type CompType = 'mod' | 'modpack' | 'resourcepack' | 'shader' | 'datapack' | 'world'

export interface ISearchHit {
  slug?: string
  title: string
  description: string
  icon_url: string
  categories: string[]
  downloads: number
  date_modified: string
  project_id: string
  project_type: CompType
  versions?: string[]
  loaders?: string[]
}

export interface IProject {
  slug: string
  id: string
  title: string
  description: string
  categories: string[]
  client_side: 'required' | 'optional' | 'unsupported' | 'unknown'
  server_side: 'required' | 'optional' | 'unsupported' | 'unknown'
  body: string
  status:
    | 'approved'
    | 'archived'
    | 'rejected'
    | 'draft'
    | 'unlisted'
    | 'processing'
    | 'withheld'
    | 'scheduled'
    | 'private'
    | 'unknown'
  requested_status: 'approved' | 'archived' | 'unlisted' | 'private' | 'draft' | null
  additional_categories: string[]
  issues_url: string | null
  source_url: string | null
  wiki_url: string | null
  donation_urls: string[]
  project_type: CompType
  downloads: number
  published: string
  updated: string
  versions: string[]
  loaders: string[]
  team: string
  organization: string | null
  icon_url: string
}

export interface IVersion {
  id: string
  name: string
  version_number: string
  game_versions: string[]
  loaders: string[]
  date_published: string
  version_type?: 'release' | 'beta' | 'alpha'
  files: {
    url: string
    filename: string
    primary: boolean
    size: number
    hashes?: { sha1?: string }
  }[]
}
