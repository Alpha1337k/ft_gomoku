import axios from "axios";
import { v4 as uuid } from "uuid";
import { EventEmitter } from "events";

interface MessageBody {
	subject: string;
	request_id: string | null;
	data: unknown;
}

export class WebSocketAPI {
	private ws!: WebSocket;
	private isOk = false;

	emitter = new EventEmitter();

	initWebsocket(cb: (result: boolean) => void) {
		this.ws = new WebSocket("ws://localhost:8000", "rust-websocket");

		this.ws.onclose = (ev) => {
			console.warn("WS Closed: ", ev);
			this.isOk = false;
			cb(false);
		};

		this.ws.onerror = (ev): void => {
			console.warn("WS Error: ", ev);
			this.isOk = false;
			cb(false);
		};

		this.ws.onopen = (ev) => {
			console.warn("WS Open: ", ev);
			this.isOk = true;
			this.emitter.emit("ready");
			cb(true);
		};

		this.ws.onmessage = (ev) => this.handleMessage(ev);
	}

	constructor() {}

	handleMessage(ev: MessageEvent<any>) {
		const data: MessageBody = JSON.parse(ev.data);

		const subject = `${data.subject}${data.request_id ? ":" + data.request_id : ""}`;

		this.emitter.emit(subject, data.data);
	}

	async sendMessage<T = unknown>(subject: string, message: unknown): Promise<T> {
		if (this.isOk == false) {
			const reconnectResult: boolean = await new Promise((res) => this.initWebsocket(res));
			if (reconnectResult == false) throw new Error("WS is not ready");
		}

		const request_id = uuid();

		console.log("OUT", {
			subject,
			request_id,
			data: message,
		});

		this.ws.send(
			JSON.stringify({
				subject,
				request_id,
				data: message,
			}),
		);

		return await new Promise((res, rej) => {
			const timeout = setTimeout(() => {
				this.emitter.removeAllListeners(`${subject}:${request_id}`);
				rej(new Error("Server did not respond in time."));
			}, 120_000);

			const listener = this.emitter.once(`${subject}:${request_id}`, (e) => {
				console.log("IN", e);
				clearTimeout(timeout);
				res(e);
			});
		});
	}
}
