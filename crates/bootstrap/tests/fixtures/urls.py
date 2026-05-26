from django.urls import path, re_path
from . import views

urlpatterns = [
    path('artists/', views.ArtistListView.as_view(), name='artist-list'),
    path('artists/<int:pk>/', views.ArtistDetailView.as_view(), name='artist-detail'),
    path('events/create/', views.EventCreateView.as_view(), name='event-create'),
    path('api/health/', views.health_check, name='health-check'),
    path('api/search/', views.search_artists, name='artist-search'),
    re_path(r'^legacy/artists/(?P<slug>[\w-]+)/$', views.artist_legacy, name='artist-legacy'),
]
