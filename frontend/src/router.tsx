import {
  createRouter,
  createRoute,
  createRootRoute,
  redirect,
  Outlet,
} from '@tanstack/react-router'
import { QueryClient } from '@tanstack/react-query'
import { AppLayout } from '@/features/layout/app-layout'
import { StoryModal } from '@/features/stories/story-modal'
import { LoginPage } from '@/features/auth/login-page'
import { OnboardingPage } from '@/features/auth/onboarding-page'
import { isOnboardingComplete } from '@/features/auth/onboarding-storage'
import { me } from '@/api/generated/auth/auth'

// ── Auth check ───────────────────────────────────────────────────────────────
type AuthResult = { authed: false } | { authed: true; userId: string; needsOnboarding: boolean }

async function checkAuth(): Promise<AuthResult> {
  try {
    const resp = await me({ credentials: 'include' })
    if (resp.status !== 200) return { authed: false }
    const userId = resp.data.user_id
    return { authed: true, userId, needsOnboarding: !isOnboardingComplete(userId) }
  } catch {
    return { authed: false }
  }
}

// ── Route tree ────────────────────────────────────────────────────────────────
const rootRoute = createRootRoute({
  component: () => <Outlet />,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  beforeLoad: async () => {
    const result = await checkAuth()
    if (!result.authed) throw redirect({ to: '/login' })
    if (result.needsOnboarding) throw redirect({ to: '/onboarding' })
    throw redirect({ to: '/projects/$projectSlug', params: { projectSlug: 'default' } })
  },
})

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/login',
  beforeLoad: async () => {
    const result = await checkAuth()
    if (!result.authed) return
    if (result.needsOnboarding) throw redirect({ to: '/onboarding' })
    throw redirect({ to: '/projects/$projectSlug', params: { projectSlug: 'default' } })
  },
  component: LoginPage,
})

const onboardingRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/onboarding',
  beforeLoad: async () => {
    const result = await checkAuth()
    if (!result.authed) throw redirect({ to: '/login' })
    if (!result.needsOnboarding) {
      throw redirect({ to: '/projects/$projectSlug', params: { projectSlug: 'default' } })
    }
  },
  component: OnboardingPage,
})

const projectRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/projects/$projectSlug',
  beforeLoad: async () => {
    const result = await checkAuth()
    if (!result.authed) throw redirect({ to: '/login' })
    if (result.needsOnboarding) throw redirect({ to: '/onboarding' })
  },
  component: AppLayout,
})

const storiesTableRoute = createRoute({
  getParentRoute: () => projectRoute,
  path: '/',
  component: () => null,
})

const settingsRoute = createRoute({
  getParentRoute: () => projectRoute,
  path: '/settings',
  component: () => null,
})

const storyModalRoute = createRoute({
  getParentRoute: () => projectRoute,
  path: '/stories/$storyId',
  component: StoryModal,
})

const taskDetailRoute = createRoute({
  getParentRoute: () => storyModalRoute,
  path: '/tasks/$taskId',
  component: () => null,
})

const routeTree = rootRoute.addChildren([
  indexRoute,
  loginRoute,
  onboardingRoute,
  projectRoute.addChildren([
    storiesTableRoute,
    settingsRoute,
    storyModalRoute.addChildren([taskDetailRoute]),
  ]),
])

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function createAppRouter(_queryClient: QueryClient) {
  return createRouter({ routeTree })
}

export type AppRouter = ReturnType<typeof createAppRouter>

declare module '@tanstack/react-router' {
  interface Register {
    router: AppRouter
  }
}
