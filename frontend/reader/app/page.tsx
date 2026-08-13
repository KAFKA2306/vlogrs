'use client'

import { useEffect, useState } from 'react'
import { getSupabase } from '@/lib/supabaseClient'

type Source = 'summary' | 'novel' | 'social'
type ObservationType = 'direct_quote' | 'paraphrase' | 'inferred_impression'
type SourceKind = 'raw_transcript' | 'summary_derived' | 'manual'

type Entry = {
  id: string
  date: string
  title: string
  content: string
  tags: string[] | null
  source: Source
  image_url?: string | null
  speaker_label?: string | null
  observation_type?: ObservationType
  source_kind?: SourceKind
  my_reaction?: string | null
}

type SocialRow = {
  id: string
  occurred_at: string
  speaker_label: string
  quote_text: string | null
  paraphrase: string | null
  observation_type: ObservationType
  context: string | null
  source_kind: SourceKind
  my_reaction: string | null
  speaker_is_public: boolean
}

const Tags = ({ tags }: { tags: string[] }) => (
  <div className="tags">
    {tags.map(t => <span key={t}>#{t}</span>)}
  </div>
)

const observationLabel = (type?: ObservationType) => {
  if (type === 'direct_quote') return 'DIRECT QUOTE'
  if (type === 'paraphrase') return 'PARAPHRASE'
  if (type === 'inferred_impression') return 'INFERENCE'
  return 'SOCIAL'
}

const sourceKindLabel = (kind?: SourceKind) => {
  if (kind === 'raw_transcript') return 'RAW TRANSCRIPT'
  if (kind === 'summary_derived') return 'SUMMARY-DERIVED'
  if (kind === 'manual') return 'MANUAL'
  return ''
}

export default function Page() {
  const [entries, setEntries] = useState<Entry[]>([])
  const [selected, setSelected] = useState<Entry>()
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string>()
  const [search, setSearch] = useState('')
  const [filter, setFilter] = useState<'all' | Source>('all')

  const fetchEntries = async () => {
    setLoading(true)
    setError(undefined)
    const supabase = getSupabase()
    if (!supabase) {
      setError('Supabase not configured')
      setLoading(false)
      return
    }

    try {
      const [summariesResult, novelsResult, socialResult] = await Promise.all([
        supabase
          .from('daily_entries')
          .select('id,date,title,content,tags,image_url')
          .eq('is_public', true)
          .order('date', { ascending: false })
          .limit(60),
        supabase
          .from('novels')
          .select('id,date,title,content,tags,image_url')
          .eq('is_public', true)
          .order('date', { ascending: false })
          .limit(60),
        supabase
          .from('social_observations')
          .select('id,occurred_at,speaker_label,quote_text,paraphrase,observation_type,context,source_kind,my_reaction,speaker_is_public')
          .eq('is_public', true)
          .order('occurred_at', { ascending: false })
          .limit(60)
      ])

      if (summariesResult.error) throw summariesResult.error
      if (novelsResult.error) throw novelsResult.error

      const summaries = (summariesResult.data || []).map(e => ({
        ...e,
        source: 'summary',
      } as Entry))

      const novels = (novelsResult.data || []).map(e => ({
        ...e,
        source: 'novel',
      } as Entry))

      let social: Entry[] = []
      if (socialResult.error) {
        // The Reader must remain available while the Social Mirror migration is
        // being rolled out. Diary and Novel are independent canonical views.
        console.warn('Social Mirror unavailable:', socialResult.error.message)
      } else {
        social = ((socialResult.data || []) as SocialRow[]).map(row => {
          const text = row.observation_type === 'direct_quote'
            ? row.quote_text || ''
            : row.paraphrase || ''
          const speaker = row.speaker_is_public ? row.speaker_label : '匿名'
          const title = row.observation_type === 'direct_quote'
            ? `「${text}」`
            : text
          const tags = [row.context, row.observation_type, row.source_kind]
            .filter((value): value is string => Boolean(value))

          return {
            id: `social-${row.id}`,
            date: row.occurred_at,
            title,
            content: `— ${speaker}`,
            tags,
            source: 'social',
            speaker_label: speaker,
            observation_type: row.observation_type,
            source_kind: row.source_kind,
            my_reaction: row.my_reaction,
          }
        })
      }

      const allEntries = [...summaries, ...novels, ...social].sort((a, b) =>
        new Date(b.date).getTime() - new Date(a.date).getTime()
      )

      setEntries(allEntries)
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { fetchEntries() }, [])

  const filteredEntries = entries.filter(e => {
    const query = search.toLowerCase()
    const matchesSearch = search
      ? e.title.toLowerCase().includes(query) ||
        e.content.toLowerCase().includes(query) ||
        e.my_reaction?.toLowerCase().includes(query) ||
        e.tags?.some(t => t.toLowerCase().includes(query))
      : true

    const matchesFilter = filter === 'all' || e.source === filter
    return matchesSearch && matchesFilter
  })

  return (
    <main className="page">
      <div className="wrap">
        <header className="hero">
          <h1>VRChat Auto Diary</h1>
          <p>Immersive memories — including the versions of me that other people saw.</p>
        </header>

        <div className="controls">
          <div className="search-bar">
            <input
              type="text"
              placeholder="Search memories..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="search-input"
            />
          </div>

          <div className="filter-tabs">
            <button className={`tab ${filter === 'all' ? 'active' : ''}`} onClick={() => setFilter('all')}>
              All
            </button>
            <button className={`tab ${filter === 'summary' ? 'active' : ''}`} onClick={() => setFilter('summary')}>
              Summaries
            </button>
            <button className={`tab ${filter === 'novel' ? 'active' : ''}`} onClick={() => setFilter('novel')}>
              Novels
            </button>
            <button className={`tab ${filter === 'social' ? 'active' : ''}`} onClick={() => setFilter('social')}>
              People Said
            </button>
          </div>
        </div>

        {loading && (
          <div className="list">
            {[1, 2, 3, 4].map(i => (
              <div key={i} className="entry" style={{ height: '320px', opacity: 0.5 }}></div>
            ))}
          </div>
        )}

        {error && (
          <div className="empty-state">
            <p>❌ {error}</p>
            <button className="ghost" onClick={fetchEntries}>Retry</button>
          </div>
        )}

        {!loading && !error && filteredEntries.length === 0 && (
          <div className="empty-state">
            <p>No entries found.</p>
          </div>
        )}

        {!loading && !error && filteredEntries.length > 0 && (
          <div className="list">
            {filteredEntries.map(e => (
              <div key={`${e.source}-${e.id}`} className={`entry ${e.source}`} onClick={() => setSelected(e)}>
                {e.image_url && (
                  <div className="entry-bg" style={{ backgroundImage: `url(${e.image_url})` }} />
                )}
                <div className="entry-overlay" />
                <div className="entry-content">
                  <div className="meta">
                    <span>{new Date(e.date).toLocaleDateString()}</span>
                    <span className={`badge ${e.source}`}>
                      {e.source === 'novel' ? 'NOVEL' : e.source === 'social' ? observationLabel(e.observation_type) : 'DIARY'}
                    </span>
                  </div>
                  <h3>{e.title}</h3>
                  <p className="preview">{e.content}</p>
                  {e.source === 'social' && e.my_reaction && (
                    <p className="reaction-preview">自分はどう受け取ったか: {e.my_reaction}</p>
                  )}
                  {e.tags?.length ? <Tags tags={e.tags} /> : null}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {selected && (
        <section className="overlay" onClick={() => setSelected(undefined)}>
          <article className={`sheet ${selected.source}`} onClick={e => e.stopPropagation()}>
            <header className="sheet-head">
              <div>
                <div className="meta">
                  <span>{new Date(selected.date).toLocaleDateString()}</span>
                  <span className={`badge ${selected.source}`}>
                    {selected.source === 'novel' ? 'NOVEL' : selected.source === 'social' ? observationLabel(selected.observation_type) : 'DIARY'}
                  </span>
                  {selected.source === 'social' && selected.source_kind && (
                    <span className="evidence-label">{sourceKindLabel(selected.source_kind)}</span>
                  )}
                </div>
                <h2>{selected.title}</h2>
              </div>
              <button className="ghost" onClick={() => setSelected(undefined)}>Close</button>
            </header>
            <div className="sheet-content-scroll">
              {selected.image_url && (
                <img src={selected.image_url} alt={selected.title} className="sheet-image" />
              )}

              {selected.source === 'social' ? (
                <div className="social-detail">
                  <p className="speaker-line">— {selected.speaker_label || '匿名'}</p>
                  {selected.my_reaction && (
                    <section className="reaction-box">
                      <h3>自分はどう受け取ったか</h3>
                      <p>{selected.my_reaction}</p>
                    </section>
                  )}
                  {selected.observation_type === 'inferred_impression' && (
                    <p className="evidence-note">これは直接の発言ではなく、推測として保存された印象です。</p>
                  )}
                  {selected.source_kind === 'summary_derived' && (
                    <p className="evidence-note">保存済み日記から復元した候補です。逐語引用としては扱いません。</p>
                  )}
                </div>
              ) : (
                <p style={{ whiteSpace: 'pre-wrap' }}>{selected.content}</p>
              )}

              {selected.tags?.length ? <Tags tags={selected.tags} /> : null}
            </div>
          </article>
        </section>
      )}
    </main>
  )
}
