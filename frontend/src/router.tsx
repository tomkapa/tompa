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
import { me } from '@/api/generated/auth/auth'

// ── Auth check ───────────────────────────────────────────────────────────────
async function checkAuth() {
  try {
    const resp = await me({ credentials: 'include' })
    return resp.status === 200
  } catch {
    return false
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
    const authed = await checkAuth()
    if (authed) {
      throw redirect({ to: '/projects/$projectId', params: { projectId: 'default' } })
    }
    throw redirect({ to: '/login' })
  },
})

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/login',
  beforeLoad: async () => {
    const authed = await checkAuth()
    if (authed) {
      throw redirect({ to: '/projects/$projectId', params: { projectId: 'default' } })
    }
  },
  component: LoginPage,
})

const projectRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/projects/$projectId',
  beforeLoad: async () => {
    const authed = await checkAuth()
    if (!authed) {
      throw redirect({ to: '/login' })
    }
  },
  component: AppLayout,
})

const storiesTableRoute = createRoute({
  getParentRoute: () => projectRoute,
  path: '/',
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
  projectRoute.addChildren([
    storiesTableRoute,
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
