// Camel Up Service Worker for PWA offline support
const CACHE_NAME = 'camel-up-v4';

// Files to cache for offline play
const urlsToCache = [
    '/',
    '/index.html',
    '/camel-up.js',
    '/camel-up_bg.wasm',
    '/manifest.json',
    '/icons/icon-192.png',
    '/icons/icon-512.png'
];

// Install event - cache core assets
self.addEventListener('install', event => {
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then(cache => {
                console.log('Caching app shell');
                return cache.addAll(urlsToCache);
            })
            // Don't auto-skipWaiting - wait for user to click "Update"
    );
});

// Activate event - clean up old caches
self.addEventListener('activate', event => {
    event.waitUntil(
        caches.keys().then(cacheNames => {
            return Promise.all(
                cacheNames.map(cacheName => {
                    if (cacheName !== CACHE_NAME) {
                        console.log('Removing old cache:', cacheName);
                        return caches.delete(cacheName);
                    }
                })
            );
        }).then(() => self.clients.claim())
    );
});

// Message event - handle update commands from the page
self.addEventListener('message', event => {
    if (event.data && event.data.type === 'SKIP_WAITING') {
        self.skipWaiting();
    }
});

// Fetch event - serve from cache, fallback to network
self.addEventListener('fetch', event => {
    event.respondWith(
        caches.match(event.request)
            .then(response => {
                // Return cached response if found
                if (response) {
                    return response;
                }

                // Clone the request for fetch
                const fetchRequest = event.request.clone();

                return fetch(fetchRequest).then(response => {
                    // Don't cache non-successful responses or non-GET requests
                    if (!response || response.status !== 200 || response.type !== 'basic' || event.request.method !== 'GET') {
                        return response;
                    }

                    // Clone response for caching
                    const responseToCache = response.clone();

                    caches.open(CACHE_NAME)
                        .then(cache => {
                            cache.put(event.request, responseToCache);
                        });

                    return response;
                });
            })
            .catch(() => {
                // Return a fallback for navigation requests
                if (event.request.mode === 'navigate') {
                    return caches.match('/index.html');
                }
            })
    );
});
