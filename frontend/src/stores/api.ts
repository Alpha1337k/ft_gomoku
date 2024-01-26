import axios from "axios";
import { v4 as uuid } from 'uuid';
import {EventEmitter} from 'events';

interface MessageBody {
	subject: string,
	requestId: string | null,
	data: unknown
}

export class WebSocketAPI
{
	private ws;
	isOk = false;

	emitter = new EventEmitter();
	
	constructor() {
		this.ws = new WebSocket("ws://localhost:8000", "rust-websocket");

		this.ws.onclose = ((ev) => {
			console.warn("WS Closed: ", ev);
			this.isOk = false;
		});

		this.ws.onerror = ((ev): void => {
			console.warn("WS Error: ", ev);
			this.isOk = false;
		})

		this.ws.onopen = ((ev) => {
			console.warn("WS Open: ", ev);
			this.isOk = true;
		})

		this.ws.onmessage = ((ev) => this.handleMessage(ev));
	}

	handleMessage(ev: MessageEvent<any>) {
		const data: MessageBody = JSON.parse(ev.data);
	
		this.emitter.emit(`${data.subject}${data.requestId ? ":" + data.requestId : ''}`, data.data);
	}

	async sendMessage<T = unknown>(subject: string, message: unknown): Promise<T> {
		if (this.isOk == false) {
			throw new Error("WS is not ready");
		}

		const requestId = uuid();

		this.ws.send(JSON.stringify({
			subject,
			requestId,
			data: message
		}));

		return await new Promise((res, rej) => {

			const timeout = setTimeout(() => {
				this.emitter.removeAllListeners(`${subject}:${requestId}`);
				rej(new Error("Server did not respond in time."));
			}, 10_000);

			const listener = this.emitter.once(`${subject}:${requestId}`, (e) => {
				clearTimeout(timeout);
				res(JSON.parse(e).data);
			})
		})
	}
}

export const ws = new WebSocketAPI();