export interface QaQuestionOption {
  label: string
  pros: string
  cons: string
}

export interface QaQuestion {
  id: string
  domain: string
  text: string
  rationale: string
  options: QaQuestionOption[]
  recommendedIndex: number
  /** Index of the selected predefined option, if answered */
  answeredIndex?: number
  /** Free-form "other" answer text, if answered with custom input */
  answeredText?: string
  /** UUID of the assigned org member, if any */
  assignedTo?: string
}

export interface OrgMember {
  user_id: string
  display_name: string
  avatar_url?: string | null
  role: string
}

export interface AppliedPattern {
  id: string
  domain: string
  pattern: string
  confidence: number
  override_count: number
}

export interface QaRound {
  id: string
  roundNumber: number
  status: 'active' | 'superseded'
  questions: QaQuestion[]
  /** Whether this round is the rollback point (restored-to marker) */
  isRollbackPoint?: boolean
  /** Number of decision patterns that were injected for this round's generation */
  appliedPatternCount?: number
  /** Summaries of injected patterns with override_count for outdated alerts */
  appliedPatterns?: AppliedPattern[]
}
