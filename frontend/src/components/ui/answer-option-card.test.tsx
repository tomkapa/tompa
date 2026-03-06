import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { AnswerOptionCard } from './answer-option-card'

const defaultProps = {
  label: 'Option A',
  pros: 'Fast to implement',
  cons: 'Less flexible',
  selected: false,
  recommended: false,
  dimmed: false,
  locked: false,
  onSelect: vi.fn(),
}

describe('AnswerOptionCard', () => {
  it('renders label correctly', () => {
    render(<AnswerOptionCard {...defaultProps} />)
    expect(screen.getByText('Option A')).toBeInTheDocument()
  })

  it('calls onSelect when radio circle clicked', () => {
    const onSelect = vi.fn()
    render(<AnswerOptionCard {...defaultProps} onSelect={onSelect} />)
    fireEvent.click(screen.getByLabelText('Select Option A'))
    expect(onSelect).toHaveBeenCalledTimes(1)
  })

  it('clicking label text expands instead of selecting', () => {
    const onSelect = vi.fn()
    render(<AnswerOptionCard {...defaultProps} onSelect={onSelect} />)
    fireEvent.click(screen.getByText('Option A'))
    expect(onSelect).not.toHaveBeenCalled()
    expect(screen.getByText('Fast to implement')).toBeInTheDocument()
  })

  it('locked+not-selected is completely inert: no radio, no expand', () => {
    const onSelect = vi.fn()
    render(<AnswerOptionCard {...defaultProps} locked={true} selected={false} onSelect={onSelect} />)
    // No radio button (non-interactive)
    expect(screen.queryByLabelText('Select Option A')).not.toBeInTheDocument()
    // Clicking does nothing — no expand
    fireEvent.click(screen.getByText('Option A'))
    expect(screen.queryByText('Fast to implement')).not.toBeInTheDocument()
    expect(onSelect).not.toHaveBeenCalled()
  })

  it('locked+selected allows expand/collapse but not reselection', () => {
    const onSelect = vi.fn()
    render(<AnswerOptionCard {...defaultProps} locked={true} selected={true} onSelect={onSelect} />)
    // Pros/cons not visible initially
    expect(screen.queryByText('Fast to implement')).not.toBeInTheDocument()
    // Clicking card expands
    fireEvent.click(screen.getByText('Option A'))
    expect(screen.getByText('Fast to implement')).toBeInTheDocument()
    // Radio click does nothing
    fireEvent.click(screen.getByLabelText('Select Option A'))
    expect(onSelect).not.toHaveBeenCalled()
  })

  it('selected state applies primary bg styling', () => {
    render(<AnswerOptionCard {...defaultProps} selected={true} />)
    const label = screen.getByText('Option A')
    const container = label.closest('[class*="bg-primary"]')
    expect(container).toBeInTheDocument()
  })

  it('default state applies background styling', () => {
    render(<AnswerOptionCard {...defaultProps} />)
    expect(screen.getByText('Option A')).toBeInTheDocument()
  })

  it('shows AI suggested badge when recommended', () => {
    render(<AnswerOptionCard {...defaultProps} recommended={true} />)
    expect(screen.getByText('AI suggested')).toBeInTheDocument()
  })

  it('does not show AI suggested badge when not recommended', () => {
    render(<AnswerOptionCard {...defaultProps} recommended={false} />)
    expect(screen.queryByText('AI suggested')).not.toBeInTheDocument()
  })

  it('expands to show pros/cons when card body clicked', () => {
    render(<AnswerOptionCard {...defaultProps} />)
    expect(screen.queryByText('Fast to implement')).not.toBeInTheDocument()
    fireEvent.click(screen.getByText('Option A'))
    expect(screen.getByText('Fast to implement')).toBeInTheDocument()
    expect(screen.getByText('Less flexible')).toBeInTheDocument()
  })

  it('collapses when card body clicked again', () => {
    render(<AnswerOptionCard {...defaultProps} />)
    fireEvent.click(screen.getByText('Option A'))
    expect(screen.getByText('Fast to implement')).toBeInTheDocument()
    fireEvent.click(screen.getByText('Option A'))
    expect(screen.queryByText('Fast to implement')).not.toBeInTheDocument()
  })

  it('selected state shows chevron and allows expand/collapse', () => {
    render(<AnswerOptionCard {...defaultProps} selected={true} />)
    // Radio button exists but clicking it does not reselect
    expect(screen.getByLabelText('Select Option A')).toBeInTheDocument()
    // Pros/cons not visible initially
    expect(screen.queryByText('Fast to implement')).not.toBeInTheDocument()
    // Clicking card expands pros/cons
    fireEvent.click(screen.getByText('Option A'))
    expect(screen.getByText('Fast to implement')).toBeInTheDocument()
    expect(screen.getByText('Less flexible')).toBeInTheDocument()
  })

  it('selected state radio click does not call onSelect', () => {
    const onSelect = vi.fn()
    render(<AnswerOptionCard {...defaultProps} selected={true} onSelect={onSelect} />)
    fireEvent.click(screen.getByLabelText('Select Option A'))
    expect(onSelect).not.toHaveBeenCalled()
  })

  it('dimmed state shows chevron and allows expand/collapse', () => {
    render(<AnswerOptionCard {...defaultProps} dimmed={true} />)
    expect(screen.queryByText('Fast to implement')).not.toBeInTheDocument()
    fireEvent.click(screen.getByText('Option A'))
    expect(screen.getByText('Fast to implement')).toBeInTheDocument()
  })
})
