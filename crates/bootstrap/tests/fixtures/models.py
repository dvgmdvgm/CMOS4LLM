from django.db import models
from django.contrib.auth.models import AbstractUser


class Artist(models.Model):
    name = models.CharField(max_length=200)
    email = models.EmailField(unique=True)
    bio = models.TextField(blank=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        ordering = ['-created_at']


class Event(models.Model):
    title = models.CharField(max_length=300)
    artist = models.ForeignKey('Artist', on_delete=models.CASCADE, related_name='events')
    venue = models.ForeignKey('Venue', on_delete=models.SET_NULL, null=True)
    date = models.DateTimeField()
    capacity = models.PositiveIntegerField(default=100)
    tags = models.ManyToManyField('Tag', blank=True)


class Venue(models.Model):
    name = models.CharField(max_length=200)
    address = models.TextField()
    capacity = models.PositiveIntegerField()


class Tag(models.Model):
    name = models.CharField(max_length=50, unique=True)


class CustomUser(AbstractUser):
    phone = models.CharField(max_length=20, blank=True)
    avatar = models.ImageField(upload_to='avatars/', null=True)
