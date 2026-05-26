from django.core.management.base import BaseCommand


class Command(BaseCommand):
    help = 'Import artists from CSV file'

    def add_arguments(self, parser):
        parser.add_argument('csv_file', type=str)
        parser.add_argument('--dry-run', action='store_true')

    def handle(self, *args, **options):
        csv_file = options['csv_file']
        self.stdout.write(f'Importing from {csv_file}')
