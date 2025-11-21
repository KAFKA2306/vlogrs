'use client'

import { useEffect, useState } from 'react'
import { supabase } from '@/lib/supabaseClient'

type Entry = { id: string; date: string; title: string; content: string; tags: string[] | null }

const colors = {
  bg: '#0b1220',
  card: '#0f172a',
  border: '#1f2937',
  text: '#e5e7eb',
  muted: '#9ca3af',
}

export default function Page() {
  const [entries, setEntries] = useState<Entry[] | null>(null)
  const [selected, setSelected] = useState<Entry | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    ;(async () => {
      const { data, error } = await supabase
        .from('daily_entries')
        .select('id,date,title,content,tags')
        .eq('is_public', true)
        .order('date', { ascending: false })
        .limit(100)
      if (error) {
        setError(error.message)
        setEntries([])
        return
      }
      setEntries((data as Entry[]) ?? [])
    })()
  }, [])

  return (
    <main
      style={{
        minHeight: '100vh',
        background: colors.bg,
        color: colors.text,
        display: 'flex',
        justifyContent: 'center',
        padding: '24px 16px',
      }}
    >
      <div style={{ width: '100%', maxWidth: 640, display: 'flex', flexDirection: 'column', gap: 12 }}>
        <h1 style={{ fontSize: 20, fontWeight: 600 }}>KAFKA Log</h1>

        {error ? (
          <div style={{ fontSize: 13, color: '#fca5a5', border: `1px solid ${colors.border}`, background: '#1b0f16', padding: 8, borderRadius: 12 }}>
            {error}
          </div>
        ) : null}

        {entries === null ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {[...Array(3)].map((_, i) => (
              <div
                key={i}
                style={{
                  height: 64,
                  borderRadius: 16,
                  background: colors.border,
                  opacity: 0.4,
                  animation: 'pulse 1.5s infinite',
                }}
              />
            ))}
          </div>
        ) : entries.length === 0 ? (
          <div style={{ fontSize: 14, color: colors.muted }}>まだ日記がありません。</div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {entries.map((e) => (
              <button
                key={e.id}
                onClick={() => setSelected(e)}
                style={{
                  textAlign: 'left',
                  width: '100%',
                  borderRadius: 16,
                  border: `1px solid ${colors.border}`,
                  background: colors.card,
                  padding: '12px 14px',
                  color: colors.text,
                }}
              >
                <div style={{ fontSize: 12, color: colors.muted }}>{new Date(e.date).toLocaleDateString()}</div>
                <div style={{ fontWeight: 600, marginTop: 2, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                  {e.title}
                </div>
                <div className="clamp-2" style={{ fontSize: 12, color: colors.muted, marginTop: 4 }}>
                  {e.content}
                </div>
                {e.tags?.length ? (
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4, marginTop: 8 }}>
                    {e.tags.map((t) => (
                      <span
                        key={t}
                        style={{
                          fontSize: 10,
                          padding: '2px 8px',
                          borderRadius: 999,
                          background: '#111827',
                          color: '#cbd5e1',
                          border: `1px solid ${colors.border}`,
                        }}
                      >
                        #{t}
                      </span>
                    ))}
                  </div>
                ) : null}
              </button>
            ))}
          </div>
        )}
      </div>

      {selected && (
        <div
          onClick={() => setSelected(null)}
          style={{
            position: 'fixed',
            inset: 0,
            zIndex: 30,
            background: 'rgba(0,0,0,0.6)',
            display: 'flex',
            alignItems: 'flex-end',
            justifyContent: 'center',
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              width: '100%',
              maxWidth: 640,
              maxHeight: '90vh',
              background: colors.bg,
              borderTopLeftRadius: 24,
              borderTopRightRadius: 24,
              overflow: 'hidden',
              border: `1px solid ${colors.border}`,
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', padding: '12px 16px' }}>
              <div>
                <div style={{ fontSize: 12, color: colors.muted }}>{new Date(selected.date).toLocaleDateString()}</div>
                <div style={{ fontSize: 16, fontWeight: 600 }}>{selected.title}</div>
              </div>
              <button onClick={() => setSelected(null)} style={{ fontSize: 12, textDecoration: 'underline', color: colors.text }}>
                閉じる
              </button>
            </div>
            <div
              style={{
                padding: '0 16px 16px',
                overflowY: 'auto',
                whiteSpace: 'pre-wrap',
                fontSize: 14,
                lineHeight: 1.6,
                color: colors.text,
              }}
            >
              {selected.content}
            </div>
          </div>
        </div>
      )}

      <style>{`
        @keyframes pulse { 0%,100%{opacity:.4;} 50%{opacity:.7;} }
        .clamp-2 { display:-webkit-box; -webkit-line-clamp:2; -webkit-box-orient:vertical; overflow:hidden; }
      `}</style>
    </main>
  )
}
