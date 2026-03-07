// Re-export generated types + local domain constants

export type { DecisionPatternResponse } from '@/api/generated/tompaAPI.schemas'
export type { ListPatternsParams, UpdatePatternRequest, SupersedePatternRequest } from '@/api/generated/tompaAPI.schemas'

export type PatternDomain = 'development' | 'security' | 'design' | 'business' | 'marketing'

export const PATTERN_DOMAINS: PatternDomain[] = [
  'development',
  'security',
  'design',
  'business',
  'marketing',
]
