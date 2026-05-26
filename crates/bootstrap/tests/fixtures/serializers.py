from rest_framework import serializers
from rest_framework.serializers import ModelSerializer


class ArtistSerializer(ModelSerializer):
    class Meta:
        model = Artist
        fields = ['id', 'name', 'email', 'bio']


class EventSerializer(serializers.ModelSerializer):
    artist_name = serializers.CharField(source='artist.name', read_only=True)

    class Meta:
        model = Event
        fields = ['id', 'title', 'artist', 'artist_name', 'date']


class TagSerializer(serializers.Serializer):
    name = serializers.CharField(max_length=50)
