from django.utils.deprecation import MiddlewareMixin


class RequestTimingMiddleware(MiddlewareMixin):
    def process_request(self, request):
        import time
        request._start_time = time.time()

    def process_response(self, request, response):
        import time
        duration = time.time() - getattr(request, '_start_time', time.time())
        response['X-Request-Duration'] = str(duration)
        return response


class TenantMiddleware:
    def __init__(self, get_response):
        self.get_response = get_response

    def __call__(self, request):
        request.tenant = self.resolve_tenant(request)
        return self.get_response(request)

    def resolve_tenant(self, request):
        return request.headers.get('X-Tenant-ID')
