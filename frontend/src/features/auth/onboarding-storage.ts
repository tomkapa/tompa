const key = (userId: string) => `tompa:onboarded:${userId}`

export function markOnboardingComplete(userId: string) {
  localStorage.setItem(key(userId), 'true')
}

export function isOnboardingComplete(userId: string) {
  return localStorage.getItem(key(userId)) === 'true'
}
