from django.views.generic import ListView, DetailView, CreateView
from rest_framework.viewsets import ModelViewSet
from rest_framework.decorators import api_view, action
from rest_framework.views import APIView
from rest_framework.response import Response
from django.http import JsonResponse


class ArtistListView(ListView):
    model = Artist
    template_name = 'artists/list.html'
    paginate_by = 20


class ArtistDetailView(DetailView):
    model = Artist
    template_name = 'artists/detail.html'


class EventCreateView(CreateView):
    model = Event
    fields = ['title', 'date', 'venue', 'capacity']


class ArtistViewSet(ModelViewSet):
    queryset = Artist.objects.all()
    serializer_class = ArtistSerializer

    @action(detail=True, methods=['post'])
    def follow(self, request, pk=None):
        pass


class EventAPIView(APIView):
    def get(self, request):
        return Response({'events': []})

    def post(self, request):
        return Response({'created': True})


@api_view(['GET'])
def health_check(request):
    return JsonResponse({'status': 'ok'})


@api_view(['GET', 'POST'])
def search_artists(request):
    return Response([])


def plain_helper_function(x, y):
    return x + y
