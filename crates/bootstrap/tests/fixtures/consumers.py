from channels.generic.websocket import AsyncWebsocketConsumer, JsonWebsocketConsumer
import json


class ChatConsumer(AsyncWebsocketConsumer):
    async def connect(self):
        await self.accept()

    async def receive(self, text_data):
        data = json.loads(text_data)
        await self.send(text_data=json.dumps({'echo': data}))


class NotificationConsumer(JsonWebsocketConsumer):
    def connect(self):
        self.accept()

    def receive_json(self, content):
        self.send_json({'ack': True})
