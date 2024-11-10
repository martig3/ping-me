import { env } from '$env/dynamic/public';
import { redirect } from '@sveltejs/kit';
import type { LayoutLoad } from './$types';
import type { User } from '$lib/types/user';

// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;

export const load: LayoutLoad = async ({ fetch }) => {
  const userResp = await fetch(`${env.PUBLIC_BASE_API}/user/me`, {
    credentials: 'include',
  });
  if (!userResp.ok) {
    throw redirect(302, '/login');
  }
  const user: User = await userResp.json();
  return { user };
};
