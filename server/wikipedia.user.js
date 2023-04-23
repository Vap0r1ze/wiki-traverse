// ==UserScript==
// @name         Wikipedia Traversal
// @namespace    http://tampermonkey.net/
// @version      0.1
// @description  try to take over the world!
// @author       You
// @match        https://en.wikipedia.org/wiki/*
// @icon         https://www.google.com/s2/favicons?sz=64&domain=thewikigame.com
// @grant        none
// ==/UserScript==

(function () {
    'use strict';

    function getCurrent() {
        let articleEls = document.querySelectorAll('.wgg-round-challenge .wgg-article-link')
        if (!articleEls.length) articleEls = document.querySelectorAll('.wgg-side-info .link')
        return articleEls ? [...articleEls].map(e => e.innerText.replaceAll(' ', '_')) : null
    }
    function traverseJSON(type, source, target) {
        let ws = new WebSocket('ws://127.0.0.1:8085')
        return new Promise((resolve, reject) => {
            ws.onopen = () => {
                ws.onmessage = msg => {
                    ws.onmessage = null
                    let data = JSON.parse(msg.data)
                    if (data.Ok) resolve(data.Ok)
                    else reject(data.Err)
                    ws.close()
                }
                ws.send(JSON.stringify({ type, source, target }))
            }
        })
    }
    const traverse = (source, target) => traverseJSON('one', source, target)
    const traverseMany = (source, target) => traverseJSON('many', source, target)
    function getPath() {
        try {
            return JSON.parse(localStorage.traversalPath)
        } catch {
            return null
        }
    }
    let path = getPath()

    function markLinks() {
        if (!path) return
        let match = location.pathname.match(/\/wiki\/(.+)$/)
        if (!match) return

        let [, page] = match

        let idx = path.findIndex(p => p.name === page)
        if (idx === -1 || idx >= path.length - 1) return

        let nextPage = path[idx + 1]
        let queries = [nextPage.name, ...nextPage.aliases].map(s => `a[href$="/wiki/${s}"], a[href*="/wiki/${s}#"]`)
        const links = [...document.querySelectorAll(queries.join(', '))]
        for (const link of links) {
            link.style.fontSize = '125%'
            link.style.fontWeight = 800
            link.style.color = '#e91e63'
            link.dataset.autoclick = true
        }
    }

    window.setTraversalPath = async (source, target) => {
        const pathTitle = `Traversal of ${source} -> ${target}`
        console.time(pathTitle)
        const data = await traverse(source, target)
        console.timeEnd(pathTitle)
        console.log('Path: %s', data.map(p => p.name).join(' -> '))
        path = data
        localStorage.traversalPath = JSON.stringify(data)
        markLinks()
    }

    markLinks()
})();
