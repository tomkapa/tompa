import {
  createRouter,
  createRoute,
  createRootRoute,
  redirect,
  Outlet,
} from '@tanstack/react-router'
import { QueryClient } from '@tanstack/react-query'

// ── Pages (placeholders) ─────────────────────────────────────────────────────
// eslint-disable-next-line react-refresh/only-export-components
function LoginPage() {
  return <div className="p-8">Login</div>
}

// eslint-disable-next-line react-refresh/only-export-components
function ProjectPage() {
  return (
    <div className="p-8">
      <Outlet />
    </div>
  )
}

// eslint-disable-next-line react-refresh/only-export-components
function StoriesTable() {
  return <div>Stories table</div>
}

// eslint-disable-next-line react-refresh/only-export-components
function StoryModal() {
  return (
    <div>
      Story modal
      <Outlet />
    </div>
  )
}

// eslint-disable-next-line react-refresh/only-export-components
function TaskDetail() {
  return <div>Task detail</div>
}

// ── Route tree ────────────────────────────────────────────────────────────────
const rootRoute = createRootRoute({
  component: () => <Outlet />,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  beforeLoad: () => {
    throw redirect({ to: '/projects/$projectId', params: { projectId: 'default' } })
  },
})

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/login',
  component: LoginPage,
})

const projectRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/projects/$projectId',
  component: ProjectPage,
})

const storiesTableRoute = createRoute({
  getParentRoute: () => projectRoute,
  path: '/',
  component: StoriesTable,
})

const storyModalRoute = createRoute({
  getParentRoute: () => projectRoute,
  path: '/stories/$storyId',
  component: StoryModal,
})

const taskDetailRoute = createRoute({
  getParentRoute: () => storyModalRoute,
  path: '/tasks/$taskId',
  component: TaskDetail,
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
