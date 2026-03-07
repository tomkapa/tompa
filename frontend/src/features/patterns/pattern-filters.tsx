import { SelectGroup } from '@/components/ui/select'
import { PATTERN_DOMAINS, type PatternDomain } from './types'

interface PatternFiltersProps {
  domain: PatternDomain | ''
  minConfidence: number
  onDomainChange: (domain: PatternDomain | '') => void
  onMinConfidenceChange: (value: number) => void
}

const CONFIDENCE_OPTIONS = [
  { value: '0', label: 'All confidence levels' },
  { value: '0.4', label: '40%+' },
  { value: '0.5', label: '50%+' },
  { value: '0.7', label: '70%+' },
  { value: '0.9', label: '90%+' },
]

function PatternFilters({
  domain,
  minConfidence,
  onDomainChange,
  onMinConfidenceChange,
}: PatternFiltersProps) {
  return (
    <div className="flex items-end gap-3">
      <SelectGroup
        label="Domain"
        value={domain}
        onChange={(e) => onDomainChange(e.target.value as PatternDomain | '')}
        className="w-[180px]"
        options={[
          { value: '', label: 'All domains' },
          ...PATTERN_DOMAINS.map((d) => ({ value: d, label: d.charAt(0).toUpperCase() + d.slice(1) })),
        ]}
      />
      <SelectGroup
        label="Min. confidence"
        value={String(minConfidence)}
        onChange={(e) => onMinConfidenceChange(Number(e.target.value))}
        className="w-[180px]"
        options={CONFIDENCE_OPTIONS}
      />
    </div>
  )
}

export { PatternFilters }
