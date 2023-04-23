import { Server } from 'ws'
import { spawn } from 'child_process'

const server = new Server({ port: 8085 })

function traverse(cmd: string, source: string, target: string) {
    return new Promise<string>((resolve, reject) => {
        const child = spawn('./target/release/wiki-traverse.exe', [cmd, source, target])

        let out = ''
        let err = ''

        child.stdout.on('data', data => {
            out += data.toString()
        })
        child.stderr.on('data', data => {
            err += data.toString()
        })

        child.on('close', code => {
            if (code === 0) resolve(out)
            else reject(err)
        })
    })
}

server.on('connection', conn => {
    console.log('New connection')
    conn.on('close', () => console.log('Connection closed'))

    conn.on('message', message => {
        const { type, source, target } = JSON.parse(message.toString())
        if (type === 'one') {
            console.log('Traversing from', source, 'to', target)
            traverse('json', source, target)
                .then(result => conn.send(result))
                .catch(err => conn.send(JSON.stringify({ Err: err })))
        } else if (type === 'many') {
            console.log('Traversing many from', source, 'to', target)
            traverse('json_many', source, target)
                .then(result => conn.send(result))
                .catch(err => conn.send(JSON.stringify({ Err: err })))
        }
    })
})

server.on('listening', () => console.log('Server started on port 8085'))
