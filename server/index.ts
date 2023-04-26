import { Server, WebSocket } from 'ws'
import { ChildProcessWithoutNullStreams, spawn } from 'child_process'
import EventEmitter from 'events'

const server = new Server({ port: 8085 })
const emitter = new EventEmitter()

let current: {
    child: ChildProcessWithoutNullStreams,
    conn: WebSocket,
} | null = null
function spawnChild(conn: WebSocket, args: string[]) {
    if (current) {
        current.child.kill()
        current.conn.send('{"Err": "New request"}')
        current = null
    }
    let child = spawn('./target/release/wiki-traverse.exe', args)
    child.stdout.on('data', data => emitter.emit('data', data))
    child.stderr.on('data', data => emitter.emit('data', data))
    child.on('close', code => {
        emitter.emit('close', code)
        if (code != null) current = null
    })
    current = { child, conn }
}

emitter.on('data', data => {
    let text = data.toString().trim()
    if (text) current?.conn.send(text)
})
emitter.on('close', (code: number) => {
    console.log('Finished with code', code)
})

server.on('connection', conn => {
    console.log('New connection')
    conn.on('close', () => {
        console.log('Connection closed')
        if (current?.conn === conn) current.child.kill()
    })

    conn.on('message', message => {
        const { type, source, target } = JSON.parse(message.toString())
        if (type === 'one') {
            console.log('Traversing from', source, 'to', target)
            spawnChild(conn, ['json', source, target])
        } else if (type === 'many') {
            console.log('Streaming from', source, 'to', target)
            spawnChild(conn, ['json_many', source, target])
        } else if (type === "stop") {
            if (current) {
                console.log('Stop requested')
                current?.child.kill()
                current = null
            }
        }
    })
})

server.on('listening', () => console.log('Server started on port 8085'))
