self.addEventListener('install', function(event) {
  event.waitUntil(preLoad());
});

const cacheName = 'cache-5ht2-2.0.0'

const preLoad = async function () {
  console.log('[PWA Builder] Install Event processing');
  return caches.open(cacheName).then(function (cache) {
    return cache.addAll(['/static/gruvbox.css', '/', '/static/elm.js']);
  });
};

self.addEventListener('fetch', function (event) {
  if (event.request.cache === 'only-if-cached' && event.request.mode !== 'same-origin') {
    return;
  }
  event.respondWith(checkResponse(event.request).catch(function () {
    return returnFromCache(event.request);
  }));
  event.waitUntil(addToCache(event.request));
});

const checkResponse = async function (request) {
  return new Promise(function (fulfil, reject) {
    fetch(request).then(function (response) {
      if (response.status !== 404) {
        fulfil(response);
      } else {
        reject();
      }
    }, reject);
  });
};

const addToCache = async function (request) {
  return caches.open(cacheName).then(async function (cache) {
    return fetch(request).then(function (response) {
      return cache.put(request, response);
    });
  });
};

const returnFromCache = async function (request) {
  return caches.open(cacheName).then(async function (cache) {
    return cache.match(request).then(function (matching) {
      if (!matching || matching.status == 404) {
        return cache.match('offline.html');
      } else {
        return matching;
      }
    });
  });
};
