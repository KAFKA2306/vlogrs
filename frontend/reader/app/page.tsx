'use client'

import { useEffect, useState } from 'react'
import { getSupabase } from '@/lib/supabaseClient'

type Entry = { id: string; date: string; title: string; content: string; tags: string[] | null }

export default function Page() {
  const [entries, setEntries] = useState<Entry[]>()
  const [selected, setSelected] = useState<Entry>()
  const [error, setError] = useState<string>()

  useEffect(() => {
    const client = getSupabase()
    if (!client) {
      setError('Supabase env missing (.env.local)')
      setEntries([])
      return
    }
    client
      .from('daily_entries')
      .select('id,date,title,content,tags')
      .eq('is_public', true)
      .order('date', { ascending: false })
      .limit(120)
      .then(({ data, error }) => {
        if (error) setError(error.message)
        setEntries((data as Entry[]) ?? [])
      })
  }, [])

  return (
    <main className="page">
      <div className="wrap">
        <header className="hero">
          <div>
            <p className="eyebrow">VRChat Auto Diary</p>
            <h1>KAFKA Log</h1>
            <p className="muted">毎日の VR 空間をすぐ読めるミニマルビュー。</p>
          </div>
          <div className="halo" />
        </header>

        {error && <p className="banner">{error}</p>}

        {entries === undefined ? (
          <div className="stack">
            {[...Array(4)].map((_, i) => (
              <div key={i} className="shimmer" />
            ))}
          </div>
        ) : entries.length === 0 ? (
          <p className="empty">まだ日記がありません。</p>
        ) : (
          <div className="list">
            {entries.map((e) => (
              <button key={e.id} className="entry" onClick={() => setSelected(e)}>
                <div className="meta">
                  <span>{new Date(e.date).toLocaleDateString()}</span>
                  <span className="dot" />
                  <span>{(e.tags ?? []).slice(0, 2).join(' · ') || 'untagged'}</span>
                </div>
                <h3>{e.title}</h3>
                <p className="preview">{e.content}</p>
                {e.tags?.length ? (
                  <div className="tags">
                    {e.tags.map((t) => (
                      <span key={t}>#{t}</span>
                    ))}
                  </div>
                ) : null}
              </button>
            ))}
          </div>
        )}
      </div>

      {selected && (
        <section className="overlay" onClick={() => setSelected(undefined)}>
          <article className="sheet" onClick={(e) => e.stopPropagation()}>
            <header className="sheet-head">
              <div>
                <p className="muted">{new Date(selected.date).toLocaleDateString()}</p>
                <h2>{selected.title}</h2>
              </div>
              <button className="ghost" onClick={() => setSelected(undefined)}>
                閉じる
              </button>
            </header>
            <p className="content">{selected.content}</p>
            {selected.tags?.length ? (
              <div className="tags">
                {selected.tags.map((t) => (
                  <span key={t}>#{t}</span>
                ))}
              </div>
            ) : null}
          </article>
        </section>
      )}
    </main>
  )
}
