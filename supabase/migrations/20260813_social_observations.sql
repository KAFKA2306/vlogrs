-- Social Mirror: record what other people actually said about the user.
-- Keep direct quotes, paraphrases, and AI inference explicitly separate.

create table if not exists social_observations (
  id bigint generated always as identity primary key,
  occurred_at timestamptz not null,
  speaker_label text not null default 'unknown',
  speaker_confidence numeric not null default 1.0 check (speaker_confidence >= 0 and speaker_confidence <= 1),
  quote_text text,
  paraphrase text,
  observation_type text not null check (
    observation_type in ('direct_quote', 'paraphrase', 'inferred_impression')
  ),
  context text,
  source_entry_id text,
  source_session_id text,
  source_excerpt text,
  source_kind text not null default 'manual' check (
    source_kind in ('raw_transcript', 'summary_derived', 'manual')
  ),
  my_reaction text,
  is_public boolean not null default false,
  speaker_is_public boolean not null default false,
  created_at timestamptz not null default now(),
  constraint social_observations_payload_check check (
    (
      observation_type = 'direct_quote'
      and quote_text is not null
      and btrim(quote_text) <> ''
    )
    or (
      observation_type in ('paraphrase', 'inferred_impression')
      and paraphrase is not null
      and btrim(paraphrase) <> ''
    )
  )
);

create index if not exists social_observations_occurred_at_idx
  on social_observations (occurred_at desc);
create index if not exists social_observations_source_entry_idx
  on social_observations (source_entry_id);
create index if not exists social_observations_type_idx
  on social_observations (observation_type);

alter table social_observations enable row level security;

do $$
begin
  if not exists (
    select 1 from pg_policies
    where tablename = 'social_observations'
      and policyname = 'Public social observations are viewable by everyone'
  ) then
    create policy "Public social observations are viewable by everyone"
      on social_observations for select
      to anon, authenticated
      using (is_public = true);
  end if;

  if not exists (
    select 1 from pg_policies
    where tablename = 'social_observations'
      and policyname = 'Service role can do everything on social observations'
  ) then
    create policy "Service role can do everything on social observations"
      on social_observations for all
      to service_role
      using (true)
      with check (true);
  end if;
end
$$;

comment on table social_observations is
  'Social Mirror observations. Direct quotes must remain distinct from paraphrases and AI inference.';
comment on column social_observations.source_kind is
  'raw_transcript is strongest evidence; summary_derived must never be silently promoted to raw-transcript verified.';
comment on column social_observations.is_public is
  'Opt-in publication only. Defaults to false because entries may contain other people''s speech.';
