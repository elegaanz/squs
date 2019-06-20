function h(tag, props, ch) {
	const elt = document.createElement(tag)

	for (const prop in props) {
		elt[prop] = props[prop]
	}

	for (const c of ch) {
		if (typeof c === 'string') {
			elt.appendChild(document.createTextNode(c))
		} else {
			elt.appendChild(c)
		}
	}

	return elt
}

const buildComment = (comm) => h('div', { className: 'comment' }, [
	h('header', {}, [
		h('img', { src: comm.avatar, alt: `${comm.author_name}'s avatar` }, []),
		h('p', { className: 'author' }, [ comm.author_name, h('small', {}, [ `@${comm.author_fqn}` ]) ])
	]),
	(!!comm.cw ? h('main', {}, [
		h('details', {}, [
			h('summary', {}, [ comm.cw ]),
			h('p', { innerHTML: comm.content }, [])
		])
	]) : h('main', { innerHTML: comm.content }, [])),
	h('footer', {}, [
		h('small', { className: 'date' }, [ comm.date ])
	]),
	h('div', { className: 'replies' }, comm.responses.map(buildComment))
])

class Squs {
	constructor(baseUrl, token, id) {
		document.head.appendChild(h('link', { href: `https://${baseUrl}/static/css/squs.css` }, []))
		fetch(`https://${baseUrl}/api/v1/comments?article=${encodeURIComponent(window.location)}`)
			.then(r => r.json())
			.then(r => {
				const elt = h('section', { className: 'comments' }, r.map(buildComment).concat([
					h('footer', { className: 'info' }, [ 'You can use your Fediverse account to comment.' ])
				]))
				document.getElementById('squs').appendChild(elt)
			})
			.catch(e => {
				console.log(e)
				const elt = h('div', { className: 'error' }, [ 'Error while loading comments.' ])
				document.getElementById('squs').appendChild(elt)
			})
	}
}
