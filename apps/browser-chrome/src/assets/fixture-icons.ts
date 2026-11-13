// DEVELOPMENT/PREVIEW ONLY. Brand favicons from the Figma reference used by
// the design fixture. Real tabs show favicons WebKit loads from the site.
// scripts/check-release-bundle.mjs fails if these reach the app bundle.
export const FIXTURE_FAVORITE_ICONS: Record<string, string> = {
  'aws': new URL('../../../../assets/icons/favorites/aws.svg', import.meta.url).href,
  'chatgpt': new URL('../../../../assets/icons/favorites/chatgpt.svg', import.meta.url).href,
  'claude': new URL('../../../../assets/icons/favorites/claude.svg', import.meta.url).href,
  'convex': new URL('../../../../assets/icons/favorites/convex.svg', import.meta.url).href,
  'discord': new URL('../../../../assets/icons/favorites/discord.svg', import.meta.url).href,
  'github': new URL('../../../../assets/icons/favorites/github.svg', import.meta.url).href,
  'gmail': new URL('../../../../assets/icons/favorites/gmail.svg', import.meta.url).href,
  'netflix': new URL('../../../../assets/icons/favorites/netflix.svg', import.meta.url).href,
  'notion': new URL('../../../../assets/icons/favorites/notion.svg', import.meta.url).href,
  'slack': new URL('../../../../assets/icons/favorites/slack.svg', import.meta.url).href,
  'supabase': new URL('../../../../assets/icons/favorites/supabase.svg', import.meta.url).href,
  'youtube': new URL('../../../../assets/icons/favorites/youtube.svg', import.meta.url).href,
};

export const FIXTURE_FAVICONS: Record<string, string> = {
  'aliexpress': new URL('../../../../assets/icons/favicons/aliexpress.svg', import.meta.url).href,
  'antigravity': new URL('../../../../assets/icons/favicons/antigravity.svg', import.meta.url).href,
  'chatgpt': new URL('../../../../assets/icons/favicons/chatgpt.svg', import.meta.url).href,
  'facebook': new URL('../../../../assets/icons/favicons/facebook.svg', import.meta.url).href,
  'figma': new URL('../../../../assets/icons/favicons/figma.svg', import.meta.url).href,
  'folder-open': new URL('../../../../assets/icons/favicons/folder-open.svg', import.meta.url).href,
  'github': new URL('../../../../assets/icons/favicons/github.svg', import.meta.url).href,
  'google': new URL('../../../../assets/icons/favicons/google.svg', import.meta.url).href,
  'lobehub': new URL('../../../../assets/icons/favicons/lobehub.svg', import.meta.url).href,
  'lucide': new URL('../../../../assets/icons/favicons/lucide.svg', import.meta.url).href,
  'nextjs': new URL('../../../../assets/icons/favicons/nextjs.svg', import.meta.url).href,
  'nucleo': new URL('../../../../assets/icons/favicons/nucleo.svg', import.meta.url).href,
  'phosphor': new URL('../../../../assets/icons/favicons/phosphor.svg', import.meta.url).href,
  'youtube': new URL('../../../../assets/icons/favicons/youtube.svg', import.meta.url).href,
};
