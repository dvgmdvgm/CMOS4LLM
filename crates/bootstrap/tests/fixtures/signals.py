from django.db.models.signals import post_save, pre_delete
from django.dispatch import receiver


@receiver(post_save, sender='Artist')
def notify_on_artist_create(sender, instance, created, **kwargs):
    if created:
        send_welcome_email(instance)


@receiver(pre_delete, sender='Event')
def cleanup_event_resources(sender, instance, **kwargs):
    instance.cancel_notifications()


def helper_not_a_signal(x):
    return x * 2
