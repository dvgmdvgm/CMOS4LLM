from django.contrib import admin
from django.contrib.admin import ModelAdmin


class ArtistAdmin(ModelAdmin):
    list_display = ['name', 'email', 'created_at']
    search_fields = ['name', 'email']


class EventAdmin(admin.ModelAdmin):
    list_display = ['title', 'artist', 'date']
    list_filter = ['date']


class VenueInline(admin.TabularInline):
    model = Venue
    extra = 1
